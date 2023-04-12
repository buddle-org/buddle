# -*- coding: utf-8 -*-

"""
nifgen
~~~~~~

Utility for generating Rust code out of niftool's format
specification in XML format.

Run this only when changes need to be made after updating
the nifxml git submodule.

Note that as of right now, this is not fully functional
and expects the user to make slight amendments to the
generated code in order to fix build errors.

Based on original work by Nikolas Wipper, licensed under
the terms of the Mozilla Public License, v2.0.
"""

from collections import namedtuple

VersionInfo = namedtuple("VersionInfo", "major minor micro patch")

TARGET_VERSIONS = (
    VersionInfo(20, 1, 0, 3),
    VersionInfo(20, 2, 0, 7),
    VersionInfo(20, 2, 0, 8),
    VersionInfo(20, 3, 0, 9),
    VersionInfo(20, 6, 0, 0),
)
