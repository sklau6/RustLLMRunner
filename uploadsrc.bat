Format - yyyymmdd_HHMMSS
echo %DATE% %TIME%
echo mm = %date:~4,2%
echo dd = %date:~7,2%
echo yyyy = %date:~10,4%
echo Timestamp = %date:~4,2%%date:~7,2%%date:~10,4%_%time:~0,2%%time:~3,2%%time:~6,2%

git add .
git commit -m "%date:~10,4%%date:~4,2%%date:~7,2%_%time:~0,2%%time:~3,2%%time:~6,2%"
git push