@echo off
REM Output file
set OUTFILE=all.txt

REM List every single file needed by your app.
REM Add to this list as you add more files/templates.
set FILES=Cargo.toml .env ^
src\main.rs src\handlers.rs src\models.rs src\templates.rs src\boards.rs ^
templates\board.html templates\thread.html templates\error.html ^
static\landing.html static\style.css

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
