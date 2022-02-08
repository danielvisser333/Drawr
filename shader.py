import os
import pathlib
import shutil
import subprocess

def compile_shaders():
    current_dir = pathlib.Path(__file__).parent.absolute()
    shader_dir = current_dir.joinpath("./renderer/src/shaders")
    for root, _, files in os.walk(shader_dir):
        for file in files:
            if not file.endswith(".spv"):
                out_file = os.path.join(root, file + ".spv")
                subprocess.run(["glslangValidator","-V",os.path.join(root, file),"-o",out_file])
                if os.path.exists(out_file):
                    if not os.path.exists(current_dir.joinpath("./target/debug/shaders")):
                        os.mkdir(current_dir.joinpath("./target/debug/shaders"))
                    if not os.path.exists(current_dir.joinpath("./target/release/shaders")):
                        os.mkdir(current_dir.joinpath("./target/release/shaders"))
                    shutil.copy(out_file, current_dir.joinpath("./target/debug/shaders/" + file + ".spv"))
                    shutil.copy(out_file, current_dir.joinpath("./target/release/shaders/" + file + ".spv"))