@echo off
setlocal
cd /d "%~dp0"
echo Configure o IDM com 8 conexoes e sem limite de velocidade.
echo Salve as opcoes e feche o IDM antes de continuar.
echo Serao 3 rodadas por programa, em ordem alternada, com SHA-256 conferido.
echo As ISOs de teste sao apagadas apos a verificacao; resultados ficam salvos.
pause
if exist "runtime\python.exe" (
  "runtime\python.exe" compare_idm.py --nibiru "nibiru-xtreme.exe" --connections 8 --idm-connections 8
) else (
  py -3 compare_idm.py --nibiru "nibiru-xtreme.exe" --connections 8 --idm-connections 8
)
echo.
echo Envie o arquivo comparacao.json da pasta criada.
pause
