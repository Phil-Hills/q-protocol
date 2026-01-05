from setuptools import setup, find_packages

setup(
    name="q-protocol-telemetry",
    version="0.1.0",
    author="Phil Hills (Systems Architect)",
    author_email="phil@philhills.ai",
    description="A lightweight telemetry and registry protocol for autonomous agent swarms.",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    url="https://github.com/Phil-Hills/q-protocol-telemetry",
    project_urls={
        "Bug Tracker": "https://github.com/Phil-Hills/q-protocol-telemetry/issues",
        "Documentation": "https://philhills.ai/docs/q-protocol",
    },
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Topic :: System :: Systems Administration",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Intended Audience :: Developers",
        "Intended Audience :: Science/Research",
    ],
    package_dir={"": "src"},
    packages=find_packages(where="src"),
    python_requires=">=3.8",
    install_requires=[
        "typing-extensions>=4.0.0",
    ],
)
