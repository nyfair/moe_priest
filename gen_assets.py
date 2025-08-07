import glob
import os

e = lambda i: 'skel' if os.path.exists(f'assets/{i}.skel') else 'prefab'
s = glob.glob('**/*.atlas', root_dir = 'assets', recursive = True)
with open('assets/spine.txt', 'w') as fd:
  for i in s:
    i = i.replace('\\', '/').replace('.atlas', '')
    fd.writelines(f'{i}.{e(i)}\n')

s = glob.glob('**/*.book.json', root_dir = 'assets', recursive = True)
with open('assets/memory.txt', 'w') as fd:
  for i in s:
    fd.writelines(f'{i.replace('\\', '/')}\n')