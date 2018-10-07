cargo build
xcopy /y /f .\target\debug\managed-alias.exe .\dist\windows\managed-alias.exe
xcopy /y /f .\ma.bat .\dist\windows\ma.bat
pushd dist\windows
install.bat
popd