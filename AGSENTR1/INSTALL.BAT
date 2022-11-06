@ECHO OFF
CLS
if *%1 == * goto show
if *%2 == * goto show
If not exist %1 goto show
if not exist %2\NUL goto show
CLS
echo [37;0mЗ  Ы д[1mдддддддддддддддддддддддддддддддд[0mд Ы   З
echo [34;1mAGSENTR.PPE Version 1.1 [0;34m by NB6 / AEGIS Corp.
echo [37;0mЗ  Ы д[1mдддддддддддддддддддддддддддддддд[0mд Ы   З
echo.
Echo [0;36m Copying PPE file...
copy AGSENTR.PPE %2 /Y >NUL
Echo [0;36m Updating PCBTEXT...
MKPCBTXT %1 /i:418 "!%2\AGSENTR.PPE" > NUL
CLS
echo [37;0mЗ  Ы д[1mдддддддддддддддддддддддддддддддд[0mд Ы   З
echo [34;1mAGSENTR.PPE Version 1.1 [0;34m by NB6 / AEGIS Corp.
echo [37;0mЗ  Ы д[1mдддддддддддддддддддддддддддддддд[0mд Ы   З
echo.
Echo [0;36m Copying PPE file...
copy AGSENTR.PPE %2 /Y >NUL
Echo [0;36m Updating PCBTEXT...
Echo.
echo [1;36m Installation Complete
echo [0m
echo.
goto end
:show
echo [37;0mЗ  Ы д[1mдддддддддддддддддддддддддддддддд[0mд Ы   З
echo [34;1mAGSENTR.PPE Version 1.1 [0;34m by NB6 / AEGIS Corp.
echo [37;0mЗ  Ы д[1mдддддддддддддддддддддддддддддддд[0mд Ы   З
echo.
echo [36;1m Usage: INSTALL C:\PCB\GEN\PCBTEXT C:\PCB\PPE
echo [0;36m                тмммммммммммммммм╬ тмммммммм╬
echo [1;30m                Path & Name of     Your PPE's
echo                 the PCBTEXT file.  directory.
echo [0m
echo.
goto end
:end
