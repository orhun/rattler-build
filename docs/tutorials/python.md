# Writing a Python package

Writing a Python package is fairly straightforward, especially for "python-only"
packages. But it becomes really interesting when compiled extensions are
involved (we will look at this in the second example).

The following recipe uses the `noarch: python` setting to build a `noarch`
package that can be installed on any platform without modification. This is very
handy for packages that are pure Python and do not contain any compiled
extensions. Additionally, `noarch: python` packages work with a range of Python
versions (contrary to packages with compiled extensions that are tied to a
specific Python version).

```yaml title="recipe.yaml"
context:
  version: "8.1.2"

package:
  name: ipywidgets
  version: ${{ version }}

source:
  url: https://pypi.io/packages/source/i/ipywidgets/ipywidgets-${{ version }}.tar.gz
  sha256: d0b9b41e49bae926a866e613a39b0f0097745d2b9f1f3dd406641b4a57ec42c9

build:
  noarch: python # (1)!
  script: pip install . -v

requirements:
  # note that there is no build section
  host:
    - pip
    - python >=3.7
    - setuptools
    - wheel
  run:
    - comm >=0.1.3
    - ipython >=6.1.0
    - jupyterlab_widgets >=3.0.10,<3.1.0
    - python >=3.7
    - traitlets >=4.3.1
    - widgetsnbextension >=4.0.10,<4.1.0

tests:
  - python:
      imports:
        - ipywidgets # (2)!

about:
  homepage: https://github.com/ipython/ipywidgets
  license: BSD-3-Clause
  license_file: LICENSE
  summary: Jupyter Interactive Widgets
  description: |
    ipywidgets are interactive HTML widgets for Jupyter notebooks and the IPython kernel.
  documentation: https://ipywidgets.readthedocs.io/en/latest/
```

1. The `noarch: python` line tells rattler-build that this package is pure
   Python and can be one-size-fits-all. Noarch packages can be installed on any
   platform without modification which is very handy.
2. The `imports` section in the tests is used to check that the package is
   installed correctly and can be imported.


## A Python package with compiled extensions

We will build a package for `numpy` – which most definitely contains compiled code.
Since compiled code is `python` version specific, we will need to specify the
`python` version explictly. This is most easily done with a "variant config" file:

```yaml title="variant_config.yaml"
python:
  - 3.11
  - 3.12
```

This will replace any `python` found in the recipe with the versions specified in the
`variant_config.yaml` file.

```yaml title="recipe.yaml"
context:
  version: 1.26.4

package:
  name: numpy
  version: ${{ version }}

source:
  - url: https://github.com/numpy/numpy/releases/download/v${{ version }}/numpy-${{ version }}.tar.gz
    sha256: 2a02aba9ed12e4ac4eb3ea9421c420301a0c6460d9830d74a9df87efa4912010

build:
  python:
    entry_points:
      - f2py = numpy.f2py.f2py2e:main  # [win]

requirements:
  build:
    - ${{ compiler('c') }}
    - ${{ compiler('cxx') }}
  host:
    # note: variant is injected here!
    - python
    - pip
    - meson-python
    - ninja
    - pkg-config
    - python-build
    - cython
    - libblas
    - libcblas
    - liblapack
  run:
    - python
  run_exports:
    - ${{ pin_subpackage("numpy") }}

tests:
  - python:
      imports:
        - numpy
        - numpy.array_api
        - numpy.array_api.linalg
        - numpy.ctypeslib

  - script:
    - f2py -h

about:
  homepage: http://numpy.org/
  license: BSD-3-Clause
  license_file: LICENSE.txt
  summary: The fundamental package for scientific computing with Python.
  documentation: https://numpy.org/doc/stable/
  repository: https://github.com/numpy/numpy
```

The build script for unix:

```bash title="build.sh"
mkdir builddir

$PYTHON -m build -w -n -x \
    -Cbuilddir=builddir \
    -Csetup-args=-Dblas=blas \
    -Csetup-args=-Dlapack=lapack

$PYTHON -m pip install dist/numpy*.whl
```

The build script for windows:

```bat title="build.bat"
mkdir builddir

%PYTHON% -m build -w -n -x ^
    -Cbuilddir=builddir ^
    -Csetup-args=-Dblas=blas ^
    -Csetup-args=-Dlapack=lapack
if %ERRORLEVEL% neq 0 exit 1

:: `pip install dist\numpy*.whl` does not work on windows,
:: so use a loop; there's only one wheel in dist/ anyway
for /f %%f in ('dir /b /S .\dist') do (
    pip install %%f
    if %ERRORLEVEL% neq 0 exit 1
)
```

### Running the recipe
Running this recipe with the variant config file will build a a total of 2 numpy packages:

```bash
rattler-build build --recipe ./numpy \
  --variant-config ./numpy/variant_config.yaml
```

At the beginning of the build process, rattler-build will print the following message to show you the variants it found:

```txt
Found variants:

numpy-1.26.4-py311h5f8ada8_0
╭─────────────────┬───────────╮
│ Variant         ┆ Version   │
╞═════════════════╪═══════════╡
│ python          ┆ 3.11      │
│ target_platform ┆ osx-arm64 │
╰─────────────────┴───────────╯

numpy-1.26.4-py312h440f24a_0
╭─────────────────┬───────────╮
│ Variant         ┆ Version   │
╞═════════════════╪═══════════╡
│ python          ┆ 3.12      │
│ target_platform ┆ osx-arm64 │
╰─────────────────┴───────────╯
```
