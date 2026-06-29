# Running Suzaku

## Windows

In a Command/PowerShell Prompt or Windows Terminal, just run the appropriate 32-bit or 64-bit Windows binary.

### Error when trying to scan a file or directory with a space in the path

When using the built-in Command or PowerShell prompt in Windows, you may receive an error that Suzaku was not able to load any files if there is a space in your file or directory path.
In order to load the log files properly, be sure to do the following:
1. Enclose the file or directory path with double quotes.
2. If it is a directory path, make sure that you do not include a backslash for the last character.

### Characters not being displayed correctly

With the default font `Lucida Console` on Windows, various characters used in the logo and tables will not be displayed properly.
You should change the font to `Consalas` to fix this.

## Linux

You first need to make the binary executable.

```bash
chmod +x ./suzaku
```

Then run it from the Suzaku root directory:

```bash
./suzaku
```

## macOS

From Terminal or [iTerm2](https://iterm2.com/), you first need to make the binary executable.

```bash
chmod +x ./suzaku
```

Then, try to run it from the Suzaku root directory:

```bash
./suzaku
```

On the latest version of macOS, you may receive a security error when you try to run it.
Click "Cancel" and then from System Preferences, open "Security & Privacy" and from the General tab, click "Allow Anyway".
After that, try to run it again.

```bash
./suzaku
```

A warning will pop up so just click "Open".
You should now be able to run suzaku.
