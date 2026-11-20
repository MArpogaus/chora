#!/usr/bin/python3
"""Compare two tour captures (stub: red first)."""


def scene_diff(a, b):
    return []


def pixel_diff(a, b, masks=()):
    return {"same": True, "blobs": [], "fraction": 0.0}


def verdict(act, scene, pixel_same, intended, checks=None):
    return {"verdict": "same"}


def compare(base, cand, intended):
    return []
