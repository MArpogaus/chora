"""MkDocs hooks of the site build.

docs/index.md includes README.md, whose images link as docs/img/..., the path
from the repository root. In the site the images sit at img/..., next to the
page, so the strict build fails on a link it cannot find. The hook includes
the README itself, with the image paths of the site.
"""

import pathlib

INCLUDE = '--8<-- "README.md"'


def on_page_markdown(markdown, page, **_):
    if page.file.src_uri != "index.md" or INCLUDE not in markdown:
        return markdown
    readme = pathlib.Path(__file__).parent.parent.joinpath("README.md").read_text()
    return markdown.replace(INCLUDE, readme.replace("](docs/img/", "](img/"))
