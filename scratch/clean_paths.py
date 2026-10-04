import glob

for f in glob.glob('.agents/skills/agm-*/skill.md'):
    with open(f, 'r', encoding='utf-8') as fp:
        c = fp.read()
    orig = c
    c = c.replace('"D:\\\\work"', '"/workspace"')
    c = c.replace('"D:\\\\work\\\\antigravity-manager"', '"/workspace/antigravity-manager"')
    c = c.replace('"D:\\\\work\\\\antigravity-manager\\\\instances"', '"/workspace/antigravity-manager/instances"')
    c = c.replace('agm instances import D:\\work\\instances.json', 'agm instances import ./instances.json')
    c = c.replace('C:\\\\Users\\\\Administrator\\\\.antigravity_tools', '~/.antigravity_tools')
    c = c.replace('d:\\\\work\\\\my-project\\\\', './my-project/')
    c = c.replace('D:\\work\\my-project', './my-project')
    c = c.replace('D:\\work\\Antigravity-Manager\\', './')
    if c != orig:
        with open(f, 'w', encoding='utf-8', newline='\n') as fp:
            fp.write(c)
        print('Cleaned examples in:', f)
