@echo off
set "TARGET=kat"
if "%TARGET%"=="kat" (
  call :build
)
goto :eof

:build
echo Building %TARGET%
