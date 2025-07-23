import glob
import os

e = lambda i: 'skel' if os.path.exists(f'assets/{i}.skel') else 'prefab'
s = glob.glob('**/rev*.atlas', root_dir = 'assets', recursive = True)
with open('assets/scene_spine.txt', 'w') as fd:
  for i in s:
    i = i.replace('\\', '/').replace('.atlas', '')
    fd.writelines(f'{i}.{e(i)}\n')

s = glob.glob('**/pose*.atlas', root_dir = 'assets', recursive = True) + glob.glob('**/ch*.atlas', root_dir = 'assets', recursive = True)
with open('assets/pose_spine.txt', 'w') as fd:
  for i in s:
    i = i.replace('\\', '/').replace('.atlas', '')
    fd.writelines(f'{i}.{e(i)}\n')

s = glob.glob('**/*.book.json', root_dir = 'assets', recursive = True)
with open('assets/event.txt', 'w') as fd:
  for i in s:
    fd.writelines(f'{i.replace('\\', '/')}\n')