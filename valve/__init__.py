# Copyright (c) 2024 Casper Lamboo
# Cura is released under the terms of the LGPLv3 or higher.

from . import constants
import platform
from UM.i18n import i18nCatalog


catalog = i18nCatalog("curaengine_plugin_valve")


if platform.machine() in ["AMD64", "x86_64"] or (platform.machine() in ["arm64"] and platform.system() == "Darwin"):
    from . import ValvePlugin

    def getMetaData():
        return {}

    def register(app):
        return {"backend_plugin":  ValvePlugin.ValvePlugin()}
else:
    from UM.Logger import Logger

    Logger.error(
        f"{constants.name} plugin is only supported on x86_64 systems for Windows and Linux and x86_64/arm64 for macOS.")

    def getMetaData():
        return {}

    def register(app):
        return {}
