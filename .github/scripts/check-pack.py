import hashlib,json,pathlib,re,sys,tarfile
archive=pathlib.Path(sys.argv[1]); assert archive.stat().st_size<=67108864
with tarfile.open(archive,'r:gz') as t:
    members=t.getmembers(); names=set(); total=0
    assert 0<len(members)<=10000
    for m in members:
        p=pathlib.PurePosixPath(m.name)
        assert not p.is_absolute() and '..' not in p.parts and p.parts[0]=='package'
        assert m.isfile() or m.isdir()
        assert str(p) not in names and '\\' not in m.name
        names.add(str(p)); total+=m.size
        assert m.size<=16777216 and total<=67108864
        assert not any(v=='.npmrc' or v.startswith('.env') for v in p.parts)
        if m.isfile():
            text=t.extractfile(m).read().decode(errors='replace')
            patterns=[r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----',r'gh[pousr]_[A-Za-z0-9]{20,}',r'github_pat_[A-Za-z0-9_]{20,}',r'npm_[A-Za-z0-9]{20,}',r'AKIA[A-Z0-9]{16}',r'/(?:home|mnt)/',r'[A-Z]:[\\/](?:Users|Documents)[\\/]',r'https?://[^/\s:@]+:[^/\s@]+@']
            assert not any(re.search(v,text,re.I) for v in patterns),'bounded archive disclosure scan finding'
    manifest=json.load(t.extractfile('package/package.json'))
    assert ('private' not in manifest or manifest['private'] is False) and 'package/LICENSE' in names
    def exports(value):
        if isinstance(value,str):
            assert value.startswith('./') and '..' not in pathlib.PurePosixPath(value).parts and '*' not in value
            assert 'package/'+value[2:] in names
        elif isinstance(value,dict):
            for v in value.values():exports(v)
        else:assert value is None
    exports(manifest.get('exports'))
data=archive.read_bytes()
print(json.dumps({'name':manifest['name'],'version':manifest['version'],'sha256':hashlib.sha256(data).hexdigest(),'file_count':len(names),'bounded_scan':'passed; not a confidentiality proof'}))
