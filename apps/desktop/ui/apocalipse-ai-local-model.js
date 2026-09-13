(function (root, factory) {
  const api = factory();
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.ApocalipseAILocalModel = api;
})(typeof globalThis !== "undefined" ? globalThis : this, function () {
  "use strict";

  // Lightweight, offline intent model. Add real-world expressions here and
  // protect every addition with a regression test before shipping it.
  const intents = [
    { name: "update_check", exact: [
      "tem atualizacao", "ha atualizacao", "existe atualizacao", "estou na ultima versao", "qual a ultima versao", "check for updates", "is there an update", "am i up to date", "latest version", "有更新吗", "检查更新", "是最新版本吗", "最新版本",
    ] },
    { name: "current_time", exact: [
      "que horas sao", "qual e a hora", "qual o horario", "me diga as horas", "what time is it", "what is the time", "current time", "现在几点", "几点了", "现在时间",
    ] },
    { name: "acknowledgement", exact: [
      "ok", "okay", "certo", "ta certo", "esta certo", "bacana", "legal", "beleza", "entendi", "combinado", "perfeito", "otimo", "tudo bem", "tranquilo", "show",
      "all right", "alright", "got it", "understood", "sounds good", "great", "fine", "nice", "cool",
      "好", "好的", "可以", "行", "明白", "明白了", "知道了", "没问题",
    ] },
    { name: "thanks", exact: [
      "obrigado", "muito obrigado", "valeu", "agradeco", "thanks", "thank you", "thank you very much", "thx", "谢谢", "多谢", "感谢",
    ] },
    { name: "goodbye", exact: [
      "tchau", "ate mais", "ate logo", "falou", "bye", "goodbye", "see you", "see you later", "再见", "拜拜", "回头见",
    ] },
    { name: "wellbeing", exact: [
      "como voce esta", "como vai", "tudo bem com voce", "how are you", "how is it going", "你好吗", "你怎么样",
    ] },
    { name: "help", exact: [
      "me ajuda", "pode me ajudar", "preciso de ajuda", "ajuda", "help", "help me", "can you help me", "请帮帮我", "帮帮我", "我需要帮助",
    ] },
    { name: "capabilities", contains: [
      "o que voce faz", "o que voce sabe fazer", "o que voce consegue fazer", "como voce pode ajudar", "suas funcoes",
      "what can you do", "how can you help", "your capabilities", "你能做什么", "你可以做什么", "怎么帮助",
    ] },
  ];

  const clean = value => String(value || "").trim().replace(/[?!.。！？]+$/u, "").trim();
  function classify(normalizedInput) {
    const input = clean(normalizedInput);
    for (const intent of intents) {
      if (intent.exact?.includes(input)) return { intent: intent.name, confidence: 1 };
      if (intent.contains?.some(phrase => input.includes(phrase))) return { intent: intent.name, confidence: 0.95 };
    }
    return null;
  }

  return { classify, intents };
});
