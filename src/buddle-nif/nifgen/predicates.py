# -*- coding: utf-8 -*-

from enum import IntEnum, auto

from . import TARGET_VERSIONS

# Modules that are specific to Bethesda games, not used by Wizard101.
BETHESDA_MODULES = ["BSMain", "BSAnimation", "BSParticle", "BSHavok", "BSLegacy"]
# Excluded attribute names that we want to ignore.
EXCLUDED_NAMES = ["RendererID", "BoneTransform"]


class VersionRelation(IntEnum):
    LOWER = auto()
    EXACT = auto()
    HIGHER = auto()


def check_version(version, string):
    def split_version_string(string):
        if "." in string:
            parts = string.split(".")
        else:
            parts = string.split("_")
        parts[0] = parts[0].lstrip("V")
        return parts

    parts = {idx: int(val) for idx, val in enumerate(split_version_string(string))}

    major = parts.get(0, 0)
    minor = parts.get(1, 0)
    micro = parts.get(2, 0)
    patch = parts.get(3, 0)

    if major < version.major:
        return VersionRelation.LOWER
    elif major > version.major:
        return VersionRelation.HIGHER

    if minor < version.minor:
        return VersionRelation.LOWER
    elif minor > version.minor:
        return VersionRelation.HIGHER

    if micro < version.micro:
        return VersionRelation.LOWER
    elif micro > version.micro:
        return VersionRelation.HIGHER

    if patch < version.patch:
        return VersionRelation.LOWER
    elif patch > version.patch:
        return VersionRelation.HIGHER

    return VersionRelation.EXACT


def should_emit_struct(attrib):
    def check_target_version(attrib, version):
        if "since" in attrib and not "versions" in attrib:
            x = check_version(version, attrib["since"]) != VersionRelation.HIGHER
            if "until" in attrib:
                x = x and check_version(version, attrib["until"]) != VersionRelation.HIGHER
            return x
        elif "versions" in attrib and not attrib["versions"].startswith("#"):
            return any(check_version(version, ver) == VersionRelation.EXACT for ver in attrib["versions"].split(" "))

        return True

    # Wizard101 doesn't use version-specific types.
    if "versions" in attrib: return False
    # No Besthesda-specific types either.
    if "module" in attrib and attrib["module"] in BETHESDA_MODULES: return False
    # If we're an excluded name, we do not care.
    if "name" in attrib and attrib["name"] in EXCLUDED_NAMES: return False
    # Wizard101 does not use PhysX, so don't bother.
    if "name" in attrib and "physx" in attrib["name"].lower(): return False
    if "name" in attrib and attrib["name"].startswith("bhk"): return False

    return any(check_target_version(attrib, v) for v in TARGET_VERSIONS)


def should_emit_member(attrib):
    def check_target_version(attrib, version):
        if "ver1" in attrib and check_version(version, attrib["ver1"]) == VersionRelation.HIGHER:
            return False

        if "ver2" in attrib and check_version(version, attrib["ver2"]) == VersionRelation.LOWER:
            return False

        return True

    if not any(check_target_version(attrib, v) for v in TARGET_VERSIONS):
        return False

    vercond = attrib.get("vercond")
    if vercond is not None:
        # Filter out all the Bethesda-specific junk.
        if vercond != "#NISTREAM#" and not vercond.startswith("!") and not vercond.startswith("#NI") and not vercond.startswith("#BSVER# #LT"):
            return False

    # There is exactly one cond=#BSSTREAMHEADER# in the Header compound,
    # because using vercond there apparently breaks niflib.
    if "cond" in attrib and attrib["cond"] == "#BSSTREAMHEADER#":
        return False

    return True
