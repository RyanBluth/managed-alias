@ECHO OFF
chcp 65001
set VAR=""
FOR /F "delims=" %%I IN ('managed-alias.exe %1 %2 %3 %4 %5 %6 %7 %8 %9') do set VAR=%%I & ECHO %%I
IF "%VAR:~0,1%"=="*" pushd %VAR:~1%