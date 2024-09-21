@echo off
setlocal

REM Define variables
set BUILD_DIR=target\debug
set OUTPUT_DIR=F:\SteamLibrary\steamapps\common\VisionsofManaDemo\VisionsofMana\Binaries\Win64
set BINARY_NAME=multimana.dll
set PDB_NAME=multimana.pdb
set RUST_BACKTRACE=1
REM set PYO3_PYTHON=C:\Users\eusth\.rye\py\cpython@3.7.3\python.exe
set PYO3_PRINT_CONFIG=0

REM Run cargo build
cargo build

REM Check if the build was successful
if %ERRORLEVEL% NEQ 0 (
    echo Build failed!
    exit /b %ERRORLEVEL%
)

REM Check if the binary exists
if not exist "%BUILD_DIR%\%BINARY_NAME%" (
    echo Binary %BINARY_NAME% not found in %BUILD_DIR%.
    exit /b 1
)

REM Check if the PDB file exists
if not exist "%BUILD_DIR%\%PDB_NAME%" (
    echo PDB file %PDB_NAME% not found in %BUILD_DIR%.
    exit /b 1
)

REM Copy the binary to the output folder
copy "%BUILD_DIR%\%BINARY_NAME%" "%OUTPUT_DIR%\"

REM Check if the copy was successful
if %ERRORLEVEL% NEQ 0 (
    echo Failed to copy %BINARY_NAME% to %OUTPUT_DIR%.
    exit /b %ERRORLEVEL%
)

REM Copy the PDB file to the output folder
copy "%BUILD_DIR%\%PDB_NAME%" "%OUTPUT_DIR%\"

REM Check if the copy was successful
if %ERRORLEVEL% NEQ 0 (
    echo Failed to copy %PDB_NAME% to %OUTPUT_DIR%.
    exit /b %ERRORLEVEL%
)

del "%OUTPUT_DIR%\scripts" /S /Q /F
mkdir "%OUTPUT_DIR%\scripts\"
xcopy /E /I scripts "%OUTPUT_DIR%\scripts"

"%OUTPUT_DIR%\VisionsofMana-Win64-Shipping.exe"

exit /b 0exit /b 0