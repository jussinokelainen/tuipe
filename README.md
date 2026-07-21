TUIPE
=====
Simple tui typing test/trainer, with different difficulties and modes.

Installation
------------
#### Arch linux:
```bash
git clone git@github.com:jussinokelainen/tuipe.git
cd tuipe/pkgbuild/arch
makepkg -si
```

#### Manually:
```bash
git clone git@github.com:jussinokelainen/tuipe.git
cd tuipe
sudo make install
```
On macOS, you might need to do
```bash
sudo make install PREFIX=/usr/local
```
Since the default location /usr/bin will
most likely fail due to permissions.


Uninstalling:
-------------

Via your package manager, or with
```bash
sudo make uninstall
```
Remember to specify the prefix if it was specified for installation
