import shader
import pathlib
import shutil;
import os;

if __name__ == '__main__':
    current_dir = pathlib.Path(__file__).parent.absolute()
    if not os.path.isdir(current_dir.joinpath("./target")):
        os.mkdir(current_dir.joinpath("./target"))
    if not os.path.isdir(current_dir.joinpath("./target/release/")):
        os.mkdir(current_dir.joinpath("./target/release/"))
    if not os.path.isdir(current_dir.joinpath("./target/debug/")):
        os.mkdir(current_dir.joinpath("./target/debug/"))
    shader.compile_shaders()
