import glob

s = glob.glob('**/*.skel', root_dir = 'assets', recursive = True)
with open('assets/binary_spine.txt', 'w') as fd:
  for i in s:
    fd.writelines(f'{i.replace('\\', '/')}\n')

s = glob.glob('**/*.prefab', root_dir = 'assets', recursive = True)
with open('assets/json_spine.txt', 'w') as fd:
  for i in s:
    fd.writelines(f'{i.replace('\\', '/')}\n')

s = glob.glob('**/*.book.json', root_dir = 'assets', recursive = True)
with open('assets/event.txt', 'w') as fd:
  for i in s:
    fd.writelines(f'{i.replace('\\', '/')}\n')