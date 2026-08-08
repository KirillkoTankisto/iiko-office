MACOS_REQUIRED := dylibbundler python3 sips iconutil
ICON := crates/iiko-office/src/assets/logo.png
TMP := temp

VERSION := $(shell sed -n '/^\[workspace\.package\]/,/^\[/{ s/^version *= *"\([^"]*\)".*/\1/p; }' Cargo.toml)

all: build

version:
	@echo $(VERSION)

set_version:
	@test -n "$(VERSION)" || { echo >&2 "usage: make set-version V=X.Y.Z"; exit 1; }
	@sed '/^\[workspace\.package\]/,/^\[/ s/^version *= *".*"/version = "$(VERSION)"/' Cargo.toml > Cargo.toml.new && mv Cargo.toml.new Cargo.toml
	@sed -e '/^name = "iiko-api"$$/,/^version = / s/^version = ".*"/version = "$(VERSION)"/' \
	     -e '/^name = "iiko-office"$$/,/^version = / s/^version = ".*"/version = "$(VERSION)"/' \
	     Cargo.lock > Cargo.lock.new && mv Cargo.lock.new Cargo.lock
	@cd packaging/linux/arch && sed 's/^pkgver=.*/pkgver=$(VERSION)/' PKGBUILD > PKGBUILD.new && mv PKGBUILD.new PKGBUILD
	@cd packaging/linux/deb/DEBIAN && sed 's/^Version: .*/Version: $(VERSION)/' control > control.new && mv control.new control
	@echo "version set to $(VERSION)"

build_debug:
	cargo build

build_debug_cached:
	RUSTC_WRAPPER=sccache cargo build

build:
	cargo build -r

build_cached:
	RUSTC_WRAPPER=sccache cargo build -r

dmg_macos: checkdeps_macos dot_app_macos
	rm -f iikoOffice.dmg
	python3 -m venv $(TMP)/venv
	$(TMP)/venv/bin/pip install --quiet dmgbuild
	$(TMP)/venv/bin/dmgbuild \
		-s packaging/macos/dmg.py \
		-D app=$(abspath $(TMP)/iikoOffice.app) \
		-D icon=$(abspath $(TMP)/AppIcon.icns) \
		-D eula=$(abspath LICENSE.rtf) \
		"iikoOffice" iikoOffice.dmg

dot_app_macos: icons_macos
	mkdir -p $(TMP)/iikoOffice.app/Contents/{MacOS,Resources}
	cp target/release/iiko-office $(TMP)/iikoOffice.app/Contents/MacOS
	cp packaging/macos/Info.plist $(TMP)/iikoOffice.app/Contents
	cp $(TMP)/AppIcon.icns $(TMP)/iikoOffice.app/Contents/Resources
	dylibbundler -cd -b -x $(TMP)/iikoOffice.app/Contents/MacOS/iiko-office -d $(TMP)/iikoOffice.app/Contents/libs -p @executable_path/../libs

icons_macos:
	mkdir -p $(TMP)/AppIcon.iconset
	sips -z 16 16 $(ICON) --out $(TMP)/AppIcon.iconset/icon_16x16.png
	sips -z 32 32 $(ICON) --out $(TMP)/AppIcon.iconset/icon_16x16@2x.png
	sips -z 32 32 $(ICON) --out $(TMP)/AppIcon.iconset/icon_32x32.png
	sips -z 64 64 $(ICON) --out $(TMP)/AppIcon.iconset/icon_32x32@2x.png
	sips -z 128 128 $(ICON) --out $(TMP)/AppIcon.iconset/icon_128x128.png
	sips -z 256 256 $(ICON) --out $(TMP)/AppIcon.iconset/icon_128x128@2x.png
	sips -z 256 256 $(ICON) --out $(TMP)/AppIcon.iconset/icon_256x256.png
	sips -z 512 512 $(ICON) --out $(TMP)/AppIcon.iconset/icon_256x256@2x.png
	sips -z 512 512 $(ICON) --out $(TMP)/AppIcon.iconset/icon_512x512.png
	cp $(ICON) $(TMP)/AppIcon.iconset/icon_512x512@2x.png
	iconutil -c icns $(TMP)/AppIcon.iconset --output $(TMP)/AppIcon.icns

checkdeps_macos:
	@for bin in $(MACOS_REQUIRED); do \
		command -v $$bin >/dev/null 2>&1 || { echo >&2 "$$bin is not installed"; exit 1; }; \
	done

clean:
	rm -rf $(TMP)

arch:
	mkdir -p $(TMP)
	cp -r packaging/linux/arch $(TMP)
	cp target/release/iiko-office packaging/linux/iiko-office.desktop packaging/linux/iiko-office.svg $(TMP)/arch
	cd $(TMP)/arch && makepkg -g >> PKGBUILD
	cd $(TMP)/arch && makepkg -d
	cp $(TMP)/arch/iiko-office-*.pkg.tar.zst .

deb:
	mkdir -p $(TMP)/deb/iiko-office/usr/bin $(TMP)/deb/iiko-office/usr/share/applications $(TMP)/deb/iiko-office/usr/share/icons/hicolor/scalable/apps
	cp -r packaging/linux/deb/* $(TMP)/deb/iiko-office
	cp target/release/iiko-office $(TMP)/deb/iiko-office/usr/bin/iiko-office
	cp packaging/linux/iiko-office.desktop $(TMP)/deb/iiko-office/usr/share/applications/iiko-office.desktop
	cp packaging/linux/iiko-office.svg $(TMP)/deb/iiko-office/usr/share/icons/hicolor/scalable/apps/iiko-office.svg
	dpkg-deb --build --root-owner-group $(TMP)/deb/iiko-office
	cp $(TMP)/deb/iiko-office.deb .

.PHONY: all dmg_macos dot_app_macos icons_macos checkdeps_macos clean_macos arch deb build build_cached build_debug build_debug_cached version set_version
