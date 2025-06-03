from setuptools import setup, find_packages
from setuptools_rust import RustExtension

setup(
    name="food_agent",
    version="0.1.0",
    packages=find_packages(),
    rust_extensions=[RustExtension("food_agent.food_agent", "Cargo.toml")],
    zip_safe=False,
    install_requires=[
        "openai",
        "python-dotenv",
        "fastapi",
        "pydantic"
    ],
)