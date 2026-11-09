"""ctl.py NAME CONTROL_XML: a xib whose 300x200 view holds CONTROL (id="ctl-aa-bbb") pinned leading 20, top 20; compiled
by Xcode; prints Xcode's frame of the control (x, y from bottom, w, h)."""
import subprocess, sys, re, io, contextlib
sys.path.insert(0, '/tmp/ak/py'); import nibdumpfull
name, control = sys.argv[1], sys.argv[2]
xib = '''<?xml version="1.0" encoding="UTF-8"?>
<document type="com.apple.InterfaceBuilder3.Cocoa.XIB" version="3.0" toolsVersion="23504" targetRuntime="MacOSX.Cocoa" propertyAccessControl="none" useAutolayout="YES" customObjectInstantitationMethod="direct">
    <dependencies><plugIn identifier="com.apple.InterfaceBuilder.CocoaPlugin" version="23504"/></dependencies>
    <objects>
        <customObject id="-2" userLabel="File's Owner"/>
        <customObject id="-1" userLabel="First Responder" customClass="FirstResponder"/>
        <customObject id="-3" userLabel="Application" customClass="NSObject"/>
        <customView id="top-aa-bbb">
            <rect key="frame" x="0.0" y="0.0" width="300" height="200"/>
            <autoresizingMask key="autoresizingMask" flexibleMaxX="YES" flexibleMinY="YES"/>
            <subviews>
%s
            </subviews>
            <constraints>
                <constraint firstItem="ctl-aa-bbb" firstAttribute="leading" secondItem="top-aa-bbb" secondAttribute="leading" constant="20" id="c1a-aa-bbb"/>
                <constraint firstItem="ctl-aa-bbb" firstAttribute="top" secondItem="top-aa-bbb" secondAttribute="top" constant="20" id="c2a-aa-bbb"/>
            </constraints>
        </customView>
    </objects>
</document>''' % control
open('/tmp/lay/%s.xib' % name, 'w').write(xib)
r = subprocess.run(['xcrun', 'ibtool', '--compile', '/tmp/lay/%s.nib' % name, '--minimum-deployment-target', '14.0', '--target-device', 'mac', '--module', 'App', '/tmp/lay/%s.xib' % name], capture_output=True, text=True)
if 'error' in r.stdout: print(name, 'ERROR', r.stdout[-300:]); sys.exit()
buf = io.StringIO()
with contextlib.redirect_stdout(buf): nibdumpfull.dump('/tmp/lay/%s.nib' % name)
frames = re.findall(r"NS.bytes=b'(\{\{[-\d.]+, [-\d.]+\}, \{[-\d.]+, [-\d.]+\}\})'", buf.getvalue())
print(name, frames)
