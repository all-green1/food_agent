from setuptools import setup
from setuptools_rust import RustExtension

setup(
    name="food_agent",
    version="0.1.0",
    packages=["food_agent"],
    package_dir={"food_agent": "."},
    rust_extensions=[RustExtension("food_agent.food_agent", "Cargo.toml")],
    zip_safe=False,
)