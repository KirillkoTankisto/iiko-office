MACOS_REQUIRED := tar dylibbundler python3 sips iconutil
ICON := crates/iiko-office/src/assets/logo.png
TMP := temp

all: dmg_macos

dmg_macos: checkdeps_macos dot_app_macos
	rm -f iikoOffice.dmg
	python3 -m venv $(TMP)/venv
	$(TMP)/venv/bin/pip install --quiet dmgbuild
	$(TMP)/venv/bin/dmgbuild \
		-s crates/iiko-office/src/assets/dmg.py \
		-D app=$(abspath $(TMP)/iikoOffice.app) \
		-D icon=$(abspath $(TMP)/AppIcon.icns) \
		-D eula=$(abspath LICENSE.rtf) \
		"iikoOffice" iikoOffice.dmg

dot_app_macos: icons_macos
	mkdir -p $(TMP)/iikoOffice.app/Contents/{MacOS,Resources}
	cp target/release/iiko-office $(TMP)/iikoOffice.app/Contents/MacOS
	cp crates/iiko-office/src/assets/Info.plist $(TMP)/iikoOffice.app/Contents
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

clean_macos:
	rm -rf $(TMP)

.PHONY: all dmg_macos dot_app_macos icons_macos checkdeps_macos clean_macos
