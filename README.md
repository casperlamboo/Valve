# Valve

![logo](valve.png)

Valve is a plugin for the [Cura Slicer](https://github.com/Ultimaker/Cura/). The plugin allows you to set a limit on the flow rate of the extruder.

## Usage

Find the plugin in the Marketplace and install it, or download the cura packages from the release page and drag the package in to the Cura Application. After restarting Cura, you will find a new setting "Maximum Flow" under the materials tab.

As of right now Cura only supports one engine plugin per modify slot. This means that you can't use the Valve plugin in combination with any other slot that uses the "Modify G-Code Path" slot. This means that before the plugin can be used the bundled "Gradual Flow" probably needs to be disabled.

<sub>The plugin icon was made by https://www.flaticon.com/authors/nadiinko</sub>