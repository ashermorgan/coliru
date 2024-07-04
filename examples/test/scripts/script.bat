@ECHO OFF
ECHO script.bat called with %*
REM Windows doesn't support ~/, but it will be edited by copy_manifest()
ECHO script.bat called with %* > ~/log.txt
