"""Exact fixture helper copies from immutable a7fc1496 POSIX harness; no launcher side effects."""

def synthetic_framework_catalog():
    """Keep the original review IDs and add 51 authored controls for a second page."""
    controls=[{"id":"framework-a","title":"Synthetic A"},
              {"id":"framework-b","title":"Synthetic B"}]
    controls.extend({"id":f"framework-extra-{index:03}",
                     "title":f"Synthetic pagination control {index}"}
                    for index in range(1,52))
    return {"catalog":{"uuid":"22222222-2222-4222-8222-222222222222",
                       "metadata":{"title":"<img src=x onerror=alert(1)>",
                                   "last-modified":"2026-09-10T00:00:00Z",
                                   "version":"1","oscal-version":"1.2.3"},
                       "controls":controls}}

def synthetic_long_project_label():
    """Author exactly 200 ASCII scalars with literal markup and an unbroken suffix."""
    return "<script>" + "L" * 192

