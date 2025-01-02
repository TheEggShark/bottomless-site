import git
from git import Repo
import os
import glob
import shutil

def main():
    if not os.path.exists('temp'):
        os.makedirs('temp')

    try:
        repo = git.Repo('temp')
    except git.InvalidGitRepositoryError as e:
        repo = git.Repo.init('temp')
        remote = repo.create_remote("origin", "https://github.com/EggShark/bottomless-pit.git")
        remote.fetch()
        repo.create_head('main', remote.refs.main)
        repo.heads.main.set_tracking_branch(remote.refs.main)
        repo.heads.main.checkout()

    remote = repo.remotes['origin']
    remote.pull()

    os.system("cd temp; cargo build --examples --release --target wasm32-unknown-unknown")
    examples = [f[14:len(f)-3] for f in glob.glob('temp/examples/*.rs')]
    for e in examples:
        os.system(f"wasm-bindgen temp/target/wasm32-unknown-unknown/release/examples/{e}.wasm --target web --out-dir website/files/examples --no-typescript")

    x = input("Want to to nuke the temp folder? ")
    while x not in ["yes", "no", "y", "n"]:
        pass
    
    if x in ["yes", "y"]:
        shutil.rmtree("temp")

if __name__ == "__main__":
    main()