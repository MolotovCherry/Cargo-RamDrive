# Cargo-RamDrive

I've always been frustrated at the amount of wear and tear that constant building can cause on SSD's (I've even burned one out!). So here are some useful instructions on how to set up a working ram drive for cargo builds

## Windows

### Installation
Download and install the driver for [arsenal image mounter](https://arsenalrecon.com/downloads), then go to [this page](https://github.com/ArsenalRecon/Arsenal-Image-Mounter/tree/master/Command%20line%20applications) and download `aim_ll.zip`. After you install the driver, you may delete the entire arsenal image mounter application (just remember that the driver will still be installed!)

- Download files under Windows directory.
- - Place files in `Windows\Documents\PowerShell` in your `C:\<UserName>\Documents\PowerShell` folder
- Set your `TMP` and `TEMP` env vars to the ram drive path you want (e.g. `R:\Temp`)
- Open task scheduler and make a new task with the following properties:
- - Run with highest privileges
- - Trigger, runs at startup
- - Actions, start a program, set to `C:\Path\To\ramdisk.bat`
- Extract the files in zip `aim_ll.zip` to the same folder as `ramdisk.bat` is in

You can edit the size of the ramdrive, volume label, and other options by editing the command line in `ramdisk.bat`. By default, the script here says 3GB

If you do not desire to move your entire temp folder over to the ram drive, you may edit the script with either a hardcoded path, or to use a different env var

\* Note: Arsenal Image Mounter is [made by the same author as imdisk](http://www.ltr-data.se/opencode.html/#ImDisk), so it's pretty reputable
\*\* You can use any RAM software you want as the PowerShell scripts will work with anything, just as long as your software is fairly compatible (cargo will [fail to work](https://github.com/rust-lang/rust/issues/90780) on any ram disk implentations that [don't implement all fs functions](https://github.com/rust-lang/rust/pull/86447))

### Using with VSCode and RustAnalyzer
There is a way to use this with VsCode/RustAnalyzer.

- Open VsCode from an instance of powershell, where the variable is already set, and RustAnalyzer will properly use it.
- If you like to use the "Open with Code" menu item like I do, do the following:
- - Download `vscode-custom-build-launcher.exe` from the releases tab (this is just a simple vscode launcher which sets the env vars)
- - Place it somewhere and put that location in PATH
- Download the reg files under `Windows`, change the paths in `install.reg` to your correct path (make sure to double check the whole path for the icon!)
- - Run `install.reg` and `delete-vscode-context-menu.reg` (this will delete the default VsCode "open with" entry). If you see double entries later after an update/install, just re-run the `delete-vscode-context-menu.reg` file
- If you want to uninstall later on, just run `uninstall.reg`
- You can also change your shortcuts to use the launcher like so  
`C:\path\to\vscode-custom-build-launcher.exe -o "C:\Scratch"`  
For more info on the flags, just type `--help` or look at the source code (note that this sets a static build dir, so if you open multiple instances of vscode, they'll use the same build dir, which is probably not what you want to happen)


Note: You may need to add the location where `code.exe` is located to your `PATH`, such as `C:\Users\USERNAME\AppData\Local\Programs\Microsoft VS Code`. Also, VsCode updates re-add the `open with` registry keys, which is why `delete-vscode-context-menu.reg` is separate (for ease)

### How it Works
Every time you cd to a different directory in PowerShell, the script will update `CARGO_BUILD_TARGET_DIR` to always point to a unique rust target folder in your temp folder for that specific project. Folders with the same project names *do not and will not* clash due to the unique id number placed at the end.

### PowerShell Commands
- `cargo clean`: Clean out the tmp rust project target dir (e.g. `R:\Temp\rust\MyProject-1234\*`)
- `Clean-Rust`: Clean out the entire tmp rust folder (e.g. `R:\Temp\rust\*`)
- `Rust-TargetDir`: Opens explorer to the rust target dir (e.g. `R:\Temp\rust\MyProject-1234\*`)
