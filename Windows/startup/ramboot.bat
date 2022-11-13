@echo off

%~dp0\aim_ll -a -t vm -s 3G -p "/fs:ntfs /v:RamDisk /q /y" -o rw,fix,hd -m R:
