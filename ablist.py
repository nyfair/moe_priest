from urllib import request
import json
import os

old = {}
new = {}
append = []
update = []
remove = []

os.system(f'cp assets/ablist.json {os.environ["TEMP"]}/ablist.json')
with open('assets/ablist.json') as fd:
  res = json.load(fd)
  for i in res['data']:
    old[i['hash'] + i['path']] = i['crc']

headers = {
  'Content-Type': 'application/json',
}
req = request.Request('https://gapi.game-monmusu-td.net/api/asset_bundle/version', headers = headers)
resp = request.urlopen(req, '{"cvr":"1","provider":"dmm"}'.encode())
x = json.loads(resp.read())
ver = x['data']['version']
ablist = f'https://assets.game-monmusu-td.net/assetbundles/ver_{ver}/webgl_r18/ablist.json'
f = request.urlopen(ablist)
raw = f.read()
with open('assets/ablist.json', 'wb') as fd:
  fd.write(raw)
res = json.loads(raw)

for i in res['data']:
  key = f'{i["hash"]}{i["path"]}'
  asset = f'https://assets.game-monmusu-td.net/assetbundles/ver_{res["baseVersion"]}/webgl_r18/{key}'
  new[key] = i['crc']
  if key not in old:
    append.append(asset)
  if key in old and i['crc'] != old[key]:
    update.append(asset)

print('New Asset:')
for i in append:
  print(i)
print('Updated Asset:')
for i in update:
  print(i)
print('Removed Asset:')
for key in old.keys():
  if key not in new:
    remove.append(key[:-6])
print(' | '.join(remove))
