import os.path

app = defines["app"]
app_name = os.path.basename(app)

files = [app]
symlinks = {"Applications": "/Applications"}
icon = defines.get("icon")

window_rect = ((200, 120), (640, 360))
default_view = "icon-view"
icon_size = 100
text_size = 12
icon_locations = {
    app_name: (150, 170),
    "Applications": (500, 170),
}

_eula = defines.get("eula")
if _eula:
    license = {
        "default-language": "en_US",
        "licenses": {
            "en_US": _eula,
            "ru_RU": _eula,
        },
        "buttons": {
            "en_US": (
                "English", "Agree", "Disagree", "Print", "Save",
                'If You agree with the terms of this license, click "Agree" '
                'to install iikoOffice. If not, click "Disagree".',
            ),
            "ru_RU": (
                "Русский", "Принять", "Отклонить", "Печать", "Сохранить",
                'Если Вы согласны с условиями данной лицензии, нажмите "Принять", '
                'чтобы установить iikoOffice. Если нет, нажмите "Отклонить".',
            ),
        },
    }
