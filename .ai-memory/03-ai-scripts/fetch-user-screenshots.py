import urllib.request
import re
import os

links = [
    ('06-prnt-i5ebhy6nsky0', 'https://prnt.sc/i5ebHY6nSKYo'),
    ('07-prnt-qlfkj9wo-3wu', 'https://prnt.sc/QlFKJ9Wo_3WU'),
    ('08-prnt-ks6-sgt1dl8e', 'https://prnt.sc/Ks6_SGT1Dl8E'),
    ('09-prnt-t5sapdl3dzpn', 'https://prnt.sc/t5SAPdL3dZpn'),
    ('10-prnt-vz9je5dl-plu', 'https://prnt.sc/vz9Je5DL_PLu'),
    ('11-prnt-duajys19-k-5', 'https://prnt.sc/duaJYs19_k-5'),
    ('12-prnt-ehjydihtw1l0', 'https://prnt.sc/EhJYdihTw1l0')
]

out_dir = os.path.join('.ai-memory', 'assets', 'ui-responsive-and-installer')
os.makedirs(out_dir, exist_ok=True)
headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)'}

for name, url in links:
    try:
        req = urllib.request.Request(url, headers=headers)
        html = urllib.request.urlopen(req).read().decode('utf-8', errors='ignore')
        m = re.search(r'<meta\s+property=["\']og:image["\']\s+content=["\']([^"\']+)["\']', html)
        if not m:
            m = re.search(r'<meta\s+name=["\']twitter:image:src["\']\s+content=["\']([^"\']+)["\']', html)
        if m:
            img_url = m.group(1)
            img_req = urllib.request.Request(img_url, headers=headers)
            img_data = urllib.request.urlopen(img_req).read()
            ext = os.path.splitext(img_url.split('?')[0])[1] or '.png'
            file_path = os.path.join(out_dir, f'{name}{ext}')
            with open(file_path, 'wb') as f:
                f.write(img_data)
            print(f'Saved {file_path} ({len(img_data)} bytes)')
        else:
            print(f'Failed to find image tag in {url}')
    except Exception as e:
        print(f'Error downloading {url}: {e}')
