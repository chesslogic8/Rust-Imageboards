@echo off
REM Change these as needed if you have other folders/files!

REM Output file
set OUTFILE=all.txt

REM List all source and template files you want to include:
set FILES=Cargo.toml .env src\main.rs src\handlers.rs src\models.rs src\templates.rs templates\board.html templates\thread.html

REM Remove old all.txt
if exist "%OUTFILE%" del "%OUTFILE%"

REM For each file, write header and contents to all.txt
for %%F in (%FILES%) do (
    echo ===============================>>"%OUTFILE%"
    echo %%F>>"%OUTFILE%"
    echo ===============================>>"%OUTFILE%"
    type "%%F">>"%OUTFILE%"
    echo.>>"%OUTFILE%"
)

echo Done! Contents of all files have been written to %OUTFILE%
pause
