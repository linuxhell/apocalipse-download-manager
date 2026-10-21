//! Native Power/Colapse adapters shared by the desktop and the backend harness.
//! No Tauri state is required; both execution and validation use production code.
use crate::nibiru_cli::{capabilities, http_url, run_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Serialize)]
pub struct PowerFormat {
    pub selection: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerInspection {
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<f64>,
    pub is_live: bool,
    pub suggested_file_name: String,
    pub formats: Vec<PowerFormat>,
}

pub async fn extract(
    path: &Path,
    source: &str,
    suggested_name: impl Fn(&str) -> String,
) -> Result<PowerInspection, String> {
    http_url(source)?;
    let caps = capabilities(path, "nibiru-power").await?;
    if caps
        .pointer("/features/html_media_discovery")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("power_html_discovery_unsupported".into());
    }
    let value = run_json(
        path,
        &[
            "extract".into(),
            source.into(),
            "--json".into(),
            "--timeout".into(),
            "30".into(),
        ],
        Duration::from_secs(40),
    )
    .await?;
    power_inspection(&value, suggested_name)
}

fn power_inspection(
    value: &Value,
    suggested_name: impl Fn(&str) -> String,
) -> Result<PowerInspection, String> {
    if value["engine"] != "nibiru-power" || value["operation"] != "extract" {
        return Err("power_operation_mismatch".into());
    }
    let result = &value["result"];
    let sources = result["sources"]
        .as_array()
        .ok_or("power_sources_missing")?;
    let mut formats = Vec::new();
    let mut suggested = None;
    for item in sources.iter().take(256) {
        let Some(source) = item["url"].as_str().and_then(|s| http_url(s).ok()) else {
            continue;
        };
        // The alpha exposes complete single sources, not a separate-track mux contract.
        if !matches!(
            apocalipse_core::classify_url(source.as_str()),
            Some(apocalipse_core::DownloadKind::Http | apocalipse_core::DownloadKind::Hls)
        ) {
            continue;
        }
        suggested.get_or_insert_with(|| suggested_name(source.as_str()));
        formats.push(PowerFormat {
            selection: format!("power:{}", source.as_str()),
            label: format!(
                "Nibiru Power · {}",
                item["label"]
                    .as_str()
                    .or(item["media_type"].as_str())
                    .unwrap_or("media")
            ),
        });
    }
    if formats.is_empty() {
        return Err("power_no_supported_sources".into());
    }
    Ok(PowerInspection {
        title: result["title"]
            .as_str()
            .unwrap_or("Media")
            .chars()
            .take(512)
            .collect(),
        thumbnail: None,
        duration: None,
        is_live: false,
        suggested_file_name: suggested.unwrap_or_else(|| "media.mp4".into()),
        formats,
    })
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QoirInputColor {
    Bt709,
    Srgb,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum MediaOperation {
    Inspect {
        input: PathBuf,
    },
    Mp4Faststart {
        input: PathBuf,
        output: PathBuf,
    },
    FlacEncode {
        input: PathBuf,
        output: PathBuf,
    },
    FlacDecode {
        input: PathBuf,
        output: PathBuf,
    },
    QoaEncode {
        input: PathBuf,
        output: PathBuf,
    },
    QoaDecode {
        input: PathBuf,
        output: PathBuf,
    },
    QoirEncode {
        input: PathBuf,
        output: PathBuf,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        input_color: Option<QoirInputColor>,
    },
    QoirDecode {
        input: PathBuf,
        output: PathBuf,
    },
    WavCut {
        input: PathBuf,
        output: PathBuf,
        start: u64,
        frames: u64,
    },
    Y4mCut {
        input: PathBuf,
        output: PathBuf,
        start: u64,
        frames: u64,
    },
    WavGain {
        input: PathBuf,
        output: PathBuf,
        db: f64,
    },
    WavNormalize {
        input: PathBuf,
        output: PathBuf,
        peak_db: f64,
    },
    WavConcat {
        inputs: Vec<PathBuf>,
        output: PathBuf,
    },
}

/// Typed ADM operations and the Colapse capability feature each one requires.
/// Inspection is the baseline command every compatible build provides.
const COLAPSE_OPERATION_FEATURES: &[(&str, Option<&str>)] = &[
    ("inspect", None),
    ("mp4-faststart", Some("mp4_faststart")),
    ("flac-encode", Some("flac_encoding")),
    ("flac-decode", Some("flac_decoding")),
    ("qoa-encode", Some("qoa_encoding")),
    ("qoa-decode", Some("qoa_decoding")),
    ("qoir-encode", Some("qoir_encoding")),
    ("qoir-decode", Some("qoir_decoding")),
    ("wav-cut", Some("sample_exact_cut")),
    ("y4m-cut", Some("y4m_frame_exact_cut")),
    ("wav-gain", Some("gain_with_clip_report")),
    ("wav-normalize", Some("peak_normalization")),
    ("wav-concat", Some("wav_concat")),
];

fn feature_enabled(capabilities: &Value, feature: &str) -> bool {
    capabilities
        .pointer(&format!("/features/{feature}"))
        .and_then(Value::as_bool)
        == Some(true)
}

/// Operations this Colapse build advertises, in the ADM's typed vocabulary.
pub fn colapse_operations(capabilities: &Value) -> Vec<&'static str> {
    COLAPSE_OPERATION_FEATURES
        .iter()
        .filter(|(_, feature)| feature.is_none_or(|feature| feature_enabled(capabilities, feature)))
        .map(|(operation, _)| *operation)
        .collect()
}

impl MediaOperation {
    pub fn args(&self) -> Result<Vec<String>, String> {
        let text = |p: &Path| -> Result<String, String> {
            if !p.is_absolute() {
                return Err("media_path_must_be_absolute".into());
            }
            p.to_str()
                .map(str::to_owned)
                .ok_or_else(|| "media_path_not_utf8".into())
        };
        let pair = |op: &str, input: &Path, output: &Path| -> Result<Vec<String>, String> {
            if !input.is_file() {
                return Err("media_input_missing".into());
            }
            if output.exists() || output == input {
                return Err("media_output_already_exists".into());
            }
            Ok(vec![op.into(), text(input)?, text(output)?])
        };
        let mut args = match self {
            Self::Inspect { input } => {
                if !input.is_file() {
                    return Err("media_input_missing".into());
                }
                vec!["inspect".into(), text(input)?]
            }
            Self::Mp4Faststart { input, output } => pair("mp4-faststart", input, output)?,
            Self::FlacEncode { input, output } => pair("flac-encode", input, output)?,
            Self::FlacDecode { input, output } => pair("flac-decode", input, output)?,
            Self::QoaEncode { input, output } => pair("qoa-encode", input, output)?,
            Self::QoaDecode { input, output } => pair("qoa-decode", input, output)?,
            Self::QoirEncode {
                input,
                output,
                input_color,
            } => {
                let mut args = pair("qoir-encode", input, output)?;
                if let Some(color) = input_color {
                    args.extend([
                        "--input-color".into(),
                        match color {
                            QoirInputColor::Bt709 => "bt709",
                            QoirInputColor::Srgb => "srgb",
                        }
                        .into(),
                    ]);
                }
                args
            }
            Self::QoirDecode { input, output } => pair("qoir-decode", input, output)?,
            Self::WavCut {
                input,
                output,
                start,
                frames,
            }
            | Self::Y4mCut {
                input,
                output,
                start,
                frames,
            } => {
                if *frames == 0 || start.checked_add(*frames).is_none() {
                    return Err("invalid_media_range".into());
                }
                let mut args = pair(
                    if matches!(self, Self::WavCut { .. }) {
                        "wav-cut"
                    } else {
                        "y4m-cut"
                    },
                    input,
                    output,
                )?;
                args.extend([
                    "--start".into(),
                    start.to_string(),
                    "--frames".into(),
                    frames.to_string(),
                ]);
                args
            }
            Self::WavGain { input, output, db }
            | Self::WavNormalize {
                input,
                output,
                peak_db: db,
            } => {
                if !db.is_finite() || !(-120.0..=120.0).contains(db) {
                    return Err("invalid_media_gain".into());
                }
                let normalize = matches!(self, Self::WavNormalize { .. });
                let mut args = pair(
                    if normalize {
                        "wav-normalize"
                    } else {
                        "wav-gain"
                    },
                    input,
                    output,
                )?;
                args.extend([
                    if normalize { "--peak-db" } else { "--db" }.into(),
                    db.to_string(),
                ]);
                args
            }
            Self::WavConcat { inputs, output } => {
                if inputs.is_empty() || inputs.len() > 256 || output.exists() {
                    return Err("invalid_concat_inputs".into());
                }
                let mut args = vec!["wav-concat".into(), text(output)?];
                for input in inputs {
                    if !input.is_file() {
                        return Err("media_input_missing".into());
                    }
                    args.push(text(input)?);
                }
                args
            }
        };
        args.push("--json".into());
        Ok(args)
    }
}

pub async fn process_media(path: &Path, request: &MediaOperation) -> Result<Value, String> {
    // Identity/schema negotiation fails explicitly for incompatible binaries.
    let advertised = capabilities(path, "nibiru-colapse").await?;
    let args = request.args()?;
    if !colapse_operations(&advertised).contains(&args[0].as_str()) {
        return Err(format!("colapse_operation_unsupported:{}", args[0]));
    }
    let value = run_json(path, &args, Duration::from_secs(6 * 3600)).await?;
    validate_colapse_response(&value, &args[0])?;
    Ok(value)
}

fn validate_colapse_response(value: &Value, operation: &str) -> Result<(), String> {
    if value["engine"] != "nibiru-colapse" || value["operation"].as_str() != Some(operation) {
        return Err("colapse_operation_mismatch".into());
    }
    if !value["result"].is_object() {
        return Err("colapse_result_missing".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn colapse_operations_follow_the_advertised_features() {
        let caps = json!({"features": {"wav_concat": true, "flac_encoding": false, "mp4_faststart": true}});
        assert_eq!(
            colapse_operations(&caps),
            vec!["inspect", "mp4-faststart", "wav-concat"]
        );
        assert_eq!(colapse_operations(&json!({})), vec!["inspect"]);
    }

    #[test]
    fn power_conversion_rejects_wrong_operation_or_no_supported_sources() {
        let valid = json!({
            "engine": "nibiru-power", "operation": "extract",
            "result": {"sources": [{"url": "https://example.test/video.mp4"}]}
        });
        for key in ["engine", "operation"] {
            let mut wrong = valid.clone();
            wrong[key] = json!("unexpected");
            assert_eq!(
                power_inspection(&wrong, |_| "name.mp4".into()).unwrap_err(),
                "power_operation_mismatch"
            );
        }
        let unsupported = json!({
            "engine": "nibiru-power", "operation": "extract",
            "result": {"sources": [
                {"url": "https://user:password@example.test/media.mp4"},
                {"url": "https://example.test/file.torrent"},
                {"url": "file:///tmp/media.mp4"}
            ]}
        });
        assert_eq!(
            power_inspection(&unsupported, |_| "unused".into()).unwrap_err(),
            "power_no_supported_sources"
        );
    }

    #[test]
    fn colapse_requires_matching_operation_and_a_result_object() {
        let valid = json!({"engine":"nibiru-colapse","operation":"inspect","result":{}});
        assert!(validate_colapse_response(&valid, "inspect").is_ok());
        assert_eq!(
            validate_colapse_response(&valid, "wav-cut").unwrap_err(),
            "colapse_operation_mismatch"
        );
        let wrong_engine = json!({"engine":"nibiru-power","operation":"inspect","result":{}});
        assert_eq!(
            validate_colapse_response(&wrong_engine, "inspect").unwrap_err(),
            "colapse_operation_mismatch"
        );
        let missing = json!({"engine":"nibiru-colapse","operation":"inspect"});
        assert_eq!(
            validate_colapse_response(&missing, "inspect").unwrap_err(),
            "colapse_result_missing"
        );
    }

    #[test]
    fn media_arguments_reject_overflow_nonfinite_gain_and_existing_output() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.wav");
        let output = directory.path().join("output.wav");
        std::fs::write(&input, b"input").unwrap();
        for (start, frames) in [(0, 0), (u64::MAX, 1)] {
            let operation = MediaOperation::WavCut {
                input: input.clone(),
                output: output.clone(),
                start,
                frames,
            };
            assert_eq!(operation.args().unwrap_err(), "invalid_media_range");
        }
        for db in [f64::NAN, f64::INFINITY, 121.0, -121.0] {
            let operation = MediaOperation::WavGain {
                input: input.clone(),
                output: output.clone(),
                db,
            };
            assert_eq!(operation.args().unwrap_err(), "invalid_media_gain");
        }
        let relative = MediaOperation::WavGain {
            input: input.clone(),
            output: PathBuf::from("relative-output.wav"),
            db: 0.0,
        };
        assert_eq!(relative.args().unwrap_err(), "media_path_must_be_absolute");
        for color in ["bt709", "srgb"] {
            let operation: MediaOperation = serde_json::from_value(json!({
                "operation": "qoir-encode", "input": input, "output": output,
                "input_color": color
            }))
            .unwrap();
            let args = operation.args().unwrap();
            assert!(args.windows(2).any(|pair| pair == ["--input-color", color]));
        }
        for color in ["other", "--events", "srgb --events"] {
            assert!(serde_json::from_value::<MediaOperation>(json!({
                "operation": "qoir-encode", "input": input, "output": output,
                "input_color": color
            }))
            .is_err());
        }
        for request in [
            json!({"operation":"qoir-decode","input":input,"output":output,"input_color":"srgb"}),
            json!({"operation":"flac-encode","input":input,"output":output,"db":0.0}),
        ] {
            assert!(serde_json::from_value::<MediaOperation>(request).is_err());
        }
        let implicit: MediaOperation = serde_json::from_value(json!({
            "operation": "qoir-encode", "input": input, "output": output
        }))
        .unwrap();
        assert!(!implicit
            .args()
            .unwrap()
            .iter()
            .any(|arg| arg == "--input-color"));
        std::fs::write(&output, b"keep this file").unwrap();
        let occupied = MediaOperation::WavGain {
            input,
            output: output.clone(),
            db: 0.0,
        };
        assert_eq!(occupied.args().unwrap_err(), "media_output_already_exists");
        assert_eq!(std::fs::read(output).unwrap(), b"keep this file");
    }

    #[tokio::test]
    #[ignore = "requires NIBIRU_POWER_BIN pointing to the compiled native engine"]
    async fn real_power_extract_uses_the_adm_runner_and_media_conversion() {
        use tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        };
        let executable =
            PathBuf::from(std::env::var_os("NIBIRU_POWER_BIN").expect("NIBIRU_POWER_BIN"));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let page = format!("{origin}/watch");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.windows(4).any(|value| value == b"\r\n\r\n") {
                let mut bytes = [0; 1024];
                let count = stream.read(&mut bytes).await.unwrap();
                assert!(count > 0 && request.len() < 16384);
                request.extend_from_slice(&bytes[..count]);
            }
            let body = br#"<title>Adapter &amp; fixture</title>
                <video src="/clip.mp4" label="720p"></video>
                <source src="/stream.m3u8" type="application/vnd.apple.mpegurl">
                <video src="/unsupported.torrent"></video>
                <audio src="https://youtube.com/watch?v=not-requested"></audio>"#;
            stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
            stream.write_all(body).await.unwrap();
        });
        // The desktop supplies its existing filename policy; this callback
        // verifies delegation without duplicating that unrelated UI helper.
        let inspection = extract(&executable, &page, |source| format!("suggested:{source}"))
            .await
            .unwrap();
        server.await.unwrap();
        assert_eq!(inspection.title, "Adapter & fixture");
        assert_eq!(
            inspection.suggested_file_name,
            format!("suggested:{origin}/clip.mp4")
        );
        assert_eq!(inspection.formats.len(), 2);
        assert_eq!(
            inspection.formats[0].selection,
            format!("power:{origin}/clip.mp4")
        );
        assert_eq!(inspection.formats[0].label, "Nibiru Power · 720p");
        assert_eq!(
            inspection.formats[1].selection,
            format!("power:{origin}/stream.m3u8")
        );
        let ui = serde_json::to_value(inspection).unwrap();
        assert_eq!(ui["isLive"], false);
        assert!(ui["thumbnail"].is_null());
        assert!(ui["duration"].is_null());
    }

    fn wav(samples: &[i16]) -> Vec<u8> {
        let size = (samples.len() * 2) as u32;
        let mut bytes = b"RIFF".to_vec();
        bytes.extend((36 + size).to_le_bytes());
        bytes.extend(b"WAVEfmt ");
        bytes.extend(16_u32.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(1_u16.to_le_bytes());
        bytes.extend(48_000_u32.to_le_bytes());
        bytes.extend(96_000_u32.to_le_bytes());
        bytes.extend(2_u16.to_le_bytes());
        bytes.extend(16_u16.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend(size.to_le_bytes());
        for sample in samples {
            bytes.extend(sample.to_le_bytes());
        }
        bytes
    }

    fn y4m(values: &[u8]) -> Vec<u8> {
        let mut bytes = b"YUV4MPEG2 W2 H2 F25:1 Ip C420\n".to_vec();
        for &value in values {
            bytes.extend(b"FRAME\n");
            bytes.extend([value; 6]);
        }
        bytes
    }

    fn atom(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut bytes = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
        bytes.extend(kind);
        bytes.extend(payload);
        bytes
    }

    // A structural MP4 fixture for metadata relocation, not a decodable video.
    fn mp4() -> Vec<u8> {
        let mut bytes = atom(b"ftyp", b"isom\0\0\0\0");
        let offset = bytes.len() as u32 + 8;
        bytes.extend(atom(b"mdat", b"adapter-media"));
        let mut mvhd = [0; 100];
        mvhd[12..16].copy_from_slice(&1000_u32.to_be_bytes());
        mvhd[16..20].copy_from_slice(&1000_u32.to_be_bytes());
        let mut tkhd = [0; 84];
        tkhd[3] = 3;
        tkhd[12..16].copy_from_slice(&1_u32.to_be_bytes());
        tkhd[76..80].copy_from_slice(&65536_u32.to_be_bytes());
        tkhd[80..84].copy_from_slice(&65536_u32.to_be_bytes());
        let mut mdhd = [0; 24];
        mdhd[12..16].copy_from_slice(&1000_u32.to_be_bytes());
        mdhd[16..20].copy_from_slice(&1000_u32.to_be_bytes());
        let mut hdlr = [0; 24];
        hdlr[8..12].copy_from_slice(b"vide");
        let mut sample = [0; 78];
        sample[7] = 1;
        let mut stsd = vec![0, 0, 0, 0, 0, 0, 0, 1];
        stsd.extend(atom(b"avc1", &sample));
        let mut stco = vec![0, 0, 0, 0, 0, 0, 0, 1];
        stco.extend(offset.to_be_bytes());
        let mut dref = vec![0, 0, 0, 0, 0, 0, 0, 1];
        dref.extend(atom(b"url ", &[0, 0, 0, 1]));
        let tables = [atom(b"stsd", &stsd), atom(b"stco", &stco)].concat();
        let information = [
            atom(b"vmhd", &[0; 12]),
            atom(b"dinf", &atom(b"dref", &dref)),
            atom(b"stbl", &tables),
        ]
        .concat();
        let media = [
            atom(b"mdhd", &mdhd),
            atom(b"hdlr", &hdlr),
            atom(b"minf", &information),
        ]
        .concat();
        let track = [atom(b"tkhd", &tkhd), atom(b"mdia", &media)].concat();
        bytes.extend(atom(
            b"moov",
            &[atom(b"mvhd", &mvhd), atom(b"trak", &track)].concat(),
        ));
        bytes
    }

    async fn colapse_request(executable: &Path, request: Value) -> Value {
        let operation: MediaOperation = serde_json::from_value(request).unwrap();
        process_media(executable, &operation).await.unwrap()
    }

    #[tokio::test]
    #[ignore = "requires NIBIRU_COLAPSE_BIN pointing to the compiled native engine"]
    async fn real_colapse_executes_all_thirteen_typed_adm_operations() {
        let executable =
            PathBuf::from(std::env::var_os("NIBIRU_COLAPSE_BIN").expect("NIBIRU_COLAPSE_BIN"));
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("input.wav");
        let original = wav(&[-1000, 0, 1000]);
        std::fs::write(&input, &original).unwrap();
        let inspected =
            colapse_request(&executable, json!({"operation":"inspect","input":input})).await;
        assert_eq!(inspected["result"]["format"], "wav");
        assert_eq!(inspected["result"]["frames"], 3);

        let compressed = directory.path().join("encoded.flac");
        let encoded = colapse_request(
            &executable,
            json!({"operation":"flac-encode","input":input,"output":compressed}),
        )
        .await;
        assert_eq!(encoded["result"]["frames"], 3);
        assert_eq!(encoded["result"]["md5_present"], true);
        assert_eq!(encoded["result"]["md5_verified"], false);
        // Independent MD5 vector for little-endian PCM16 [-1000, 0, 1000].
        assert_eq!(encoded["result"]["md5"], "26286da2c3568dafde63da7bfab38501");
        let compressed_bytes = std::fs::read(&compressed).unwrap();
        assert_eq!(&compressed_bytes[..4], b"fLaC");
        let inspected = colapse_request(
            &executable,
            json!({"operation":"inspect","input":compressed}),
        )
        .await;
        assert_eq!(inspected["result"]["format"], "flac");
        assert_eq!(inspected["result"]["frames"], 3);
        assert_eq!(inspected["result"]["bits_per_sample"], 16);
        assert_eq!(inspected["result"]["audio_integrity_verified"], false);
        assert_eq!(inspected["result"]["md5"], encoded["result"]["md5"]);
        let decoded = directory.path().join("decoded.wav");
        let result = colapse_request(
            &executable,
            json!({"operation":"flac-decode","input":compressed,"output":decoded}),
        )
        .await;
        assert_eq!(result["result"]["frames"], 3);
        assert_eq!(result["result"]["md5_present"], true);
        assert_eq!(result["result"]["md5_verified"], true);
        assert_eq!(result["result"]["md5"], encoded["result"]["md5"]);
        assert_eq!(std::fs::read(&decoded).unwrap(), original);
        for occupied in [
            MediaOperation::FlacEncode {
                input: input.clone(),
                output: compressed.clone(),
            },
            MediaOperation::FlacDecode {
                input: compressed.clone(),
                output: decoded.clone(),
            },
        ] {
            assert_eq!(
                process_media(&executable, &occupied).await.unwrap_err(),
                "media_output_already_exists"
            );
        }
        assert_eq!(std::fs::read(&compressed).unwrap(), compressed_bytes);
        assert_eq!(std::fs::read(&decoded).unwrap(), original);

        let qoa = directory.path().join("encoded.qoa");
        let encoded_qoa = colapse_request(
            &executable,
            json!({"operation":"qoa-encode","input":input,"output":qoa}),
        )
        .await;
        assert_eq!(encoded_qoa["result"]["frames"], 3);
        assert_eq!(encoded_qoa["result"]["lossy"], true);
        assert_eq!(encoded_qoa["result"]["checksum_available"], false);
        let qoa_bytes = std::fs::read(&qoa).unwrap();
        assert_eq!(&qoa_bytes[..4], b"qoaf");
        let inspected_qoa =
            colapse_request(&executable, json!({"operation":"inspect","input":qoa})).await;
        assert_eq!(inspected_qoa["result"]["format"], "qoa");
        assert_eq!(inspected_qoa["result"]["frames"], 3);
        assert_eq!(inspected_qoa["result"]["audio_integrity_verified"], false);

        // Independent QOA vector: zero LMS state and scale/residual symbols 0.
        // The spec rounds 0.75 to +1; unchanged zero weights predict zero.
        let reference_qoa = directory.path().join("reference.qoa");
        let mut reference = b"qoaf".to_vec();
        reference.extend(3_u32.to_be_bytes());
        reference.extend([1, 0, 0xbb, 0x80, 0, 3, 0, 32]);
        reference.extend([0; 24]);
        std::fs::write(&reference_qoa, &reference).unwrap();
        let qoa_wav = directory.path().join("qoa-decoded.wav");
        let decoded_qoa = colapse_request(
            &executable,
            json!({"operation":"qoa-decode","input":reference_qoa,"output":qoa_wav}),
        )
        .await;
        assert_eq!(decoded_qoa["result"]["frames"], 3);
        assert_eq!(decoded_qoa["result"]["lossy"], true);
        assert_eq!(decoded_qoa["result"]["checksum_available"], false);
        assert_eq!(std::fs::read(&qoa_wav).unwrap(), wav(&[1, 1, 1]));
        for occupied in [
            MediaOperation::QoaEncode {
                input: input.clone(),
                output: qoa.clone(),
            },
            MediaOperation::QoaDecode {
                input: reference_qoa.clone(),
                output: qoa_wav.clone(),
            },
        ] {
            assert_eq!(
                process_media(&executable, &occupied).await.unwrap_err(),
                "media_output_already_exists"
            );
        }
        assert_eq!(std::fs::read(&qoa).unwrap(), qoa_bytes);
        assert_eq!(std::fs::read(&reference_qoa).unwrap(), reference);
        assert_eq!(std::fs::read(&qoa_wav).unwrap(), wav(&[1, 1, 1]));

        let image = directory.path().join("entrada cor 中文.pam");
        let pixels = [255, 0, 127, 0, 3, 5, 7, 255];
        let mut pam = b"P7\n# COLORSPACE SRGB\nWIDTH 2\nHEIGHT 1\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
        pam.extend(pixels);
        std::fs::write(&image, &pam).unwrap();
        let qoir = directory.path().join("imagem nativa.qoir");
        let encoded_qoir = colapse_request(
            &executable,
            json!({"operation":"qoir-encode","input":image,"output":qoir,"input_color":"srgb"}),
        )
        .await;
        assert_eq!(encoded_qoir["result"]["width"], 2);
        assert_eq!(encoded_qoir["result"]["height"], 1);
        assert_eq!(encoded_qoir["result"]["channels"], 4);
        assert_eq!(encoded_qoir["result"]["alpha_mode"], "straight");
        assert_eq!(encoded_qoir["result"]["color_space"], "srgb");
        assert_eq!(encoded_qoir["result"]["lossy"], false);
        assert_eq!(encoded_qoir["result"]["experimental"], true);
        let qoir_bytes = std::fs::read(&qoir).unwrap();
        assert_eq!(&qoir_bytes[..4], b"QOIR");
        let inspected_qoir =
            colapse_request(&executable, json!({"operation":"inspect","input":qoir})).await;
        assert_eq!(inspected_qoir["result"]["format"], "qoir");
        assert_eq!(inspected_qoir["result"]["color_space"], "srgb");
        assert_eq!(inspected_qoir["result"]["pixel_data_decoded"], false);
        assert_eq!(inspected_qoir["result"]["checksum_available"], false);

        // Independent literal QOIR tile: BGRA, straight alpha, implicit sRGB.
        // It covers fully transparent RGB values as well as an opaque pixel.
        let mut reference = Vec::new();
        for (kind, payload) in [
            (b"QOIR", [2, 0, 0, 2, 1, 0, 0, 0].as_slice()),
            (
                b"QPIX",
                [8, 0, 0, 0, 127, 0, 255, 0, 7, 5, 3, 255].as_slice(),
            ),
            (b"QEND", [].as_slice()),
        ] {
            reference.extend(kind);
            reference.extend((payload.len() as u64).to_le_bytes());
            reference.extend(payload);
        }
        let reference_qoir = directory.path().join("reference.qoir");
        std::fs::write(&reference_qoir, &reference).unwrap();
        let decoded_pam = directory.path().join("decoded.pam");
        let decoded_qoir = colapse_request(
            &executable,
            json!({"operation":"qoir-decode","input":reference_qoir,"output":decoded_pam}),
        )
        .await;
        assert_eq!(decoded_qoir["result"]["channels"], 4);
        assert_eq!(decoded_qoir["result"]["alpha_mode"], "straight");
        assert_eq!(std::fs::read(&decoded_pam).unwrap(), pam);
        for occupied in [
            MediaOperation::QoirEncode {
                input: image.clone(),
                output: qoir.clone(),
                input_color: Some(QoirInputColor::Srgb),
            },
            MediaOperation::QoirDecode {
                input: reference_qoir.clone(),
                output: decoded_pam.clone(),
            },
        ] {
            assert_eq!(
                process_media(&executable, &occupied).await.unwrap_err(),
                "media_output_already_exists"
            );
        }
        assert_eq!(std::fs::read(&image).unwrap(), pam);
        assert_eq!(std::fs::read(&qoir).unwrap(), qoir_bytes);
        assert_eq!(std::fs::read(&reference_qoir).unwrap(), reference);
        assert_eq!(std::fs::read(&decoded_pam).unwrap(), pam);

        let cut = directory.path().join("cut.wav");
        let result = colapse_request(
            &executable,
            json!({"operation":"wav-cut","input":input,"output":cut,"start":1,"frames":2}),
        )
        .await;
        assert_eq!(result["result"]["frames"], 2);
        assert_eq!(std::fs::read(&cut).unwrap(), wav(&[0, 1000]));

        let gain = directory.path().join("gain.wav");
        colapse_request(
            &executable,
            json!({"operation":"wav-gain","input":input,"output":gain,"db":0.0}),
        )
        .await;
        assert_eq!(std::fs::read(gain).unwrap(), original);

        let normalized = directory.path().join("normalized.wav");
        colapse_request(
            &executable,
            json!({"operation":"wav-normalize","input":input,"output":normalized,"peak_db":0.0}),
        )
        .await;
        assert_eq!(std::fs::read(normalized).unwrap(), wav(&[-32767, 0, 32767]));

        let concatenated = directory.path().join("concat.wav");
        colapse_request(
            &executable,
            json!({"operation":"wav-concat","inputs":[input,input],"output":concatenated}),
        )
        .await;
        assert_eq!(
            std::fs::read(concatenated).unwrap(),
            wav(&[-1000, 0, 1000, -1000, 0, 1000])
        );

        let video = directory.path().join("input.y4m");
        let video_cut = directory.path().join("cut.y4m");
        std::fs::write(&video, y4m(&[10, 20, 30])).unwrap();
        let result = colapse_request(
            &executable,
            json!({"operation":"y4m-cut","input":video,"output":video_cut,"start":1,"frames":1}),
        )
        .await;
        assert_eq!(result["result"]["frames"], 1);
        assert_eq!(std::fs::read(video_cut).unwrap(), y4m(&[20]));

        let movie = directory.path().join("input.mp4");
        let faststart = directory.path().join("faststart.mp4");
        let movie_bytes = mp4();
        std::fs::write(&movie, &movie_bytes).unwrap();
        let result = colapse_request(
            &executable,
            json!({"operation":"mp4-faststart","input":movie,"output":faststart}),
        )
        .await;
        assert_eq!(result["result"]["relocated"], true);
        assert_eq!(result["result"]["adjusted_offsets"], 1);
        let moved = std::fs::read(faststart).unwrap();
        assert_eq!(moved.len(), movie_bytes.len());
        let moov = moved.windows(4).position(|value| value == b"moov").unwrap();
        let mdat = moved.windows(4).position(|value| value == b"mdat").unwrap();
        assert!(moov < mdat);
        assert_eq!(&moved[mdat + 4..], b"adapter-media");

        let occupied = MediaOperation::WavGain {
            input: input.clone(),
            output: cut.clone(),
            db: 0.0,
        };
        assert_eq!(
            process_media(&executable, &occupied).await.unwrap_err(),
            "media_output_already_exists"
        );
        assert_eq!(std::fs::read(cut).unwrap(), wav(&[0, 1000]));
        assert_eq!(std::fs::read(input).unwrap(), original);
    }
}
