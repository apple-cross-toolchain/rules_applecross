from probe import *
def s1(t=0, a=1.0, b=-1.0): return [B(t,0,0,3.0,1,2), L(t,1,a), L(t,2,b)]
def s3(t=0): return [B(t,0,0,3.0,1,2), L(t,1,[(0,1.0)]), L(t,2,[(2,1.0)])]
cls('C2s', s1(), ['a','b'])
cls('C2s_tf2', s1(), ['a','b'], transform=2)
cls('C2i', s1(), [5, 7])
cls('C2i_neg', s1(), [-5, 7000000000])
cls('C3s', s3(), ['lo','mid','hi'])
cls('C3s_tf1', s3(), ['lo','mid','hi'], transform=1)
cls('C3s_tf3', s3(), ['lo','mid','hi'], transform=3)
cls('C3i', s3(), [1, 2, 3])
cls('C3s_noprob', s3(), ['lo','mid','hi'], probname=None)
cls('C3s_names', s3(), ['lo','mid','hi'], probname='P', outname='cls')
cls('C2s_npd2', [B(0,0,0,3.0,1,2), L(0,1,[1.0,0.0]), L(0,2,[0.0,1.0])], ['a','b'], npd=2)
cls('C2s_leaf', [L(0,0,0.25)], ['a','b'])
cls('C1s', s1(), ['only'], npd=1)
cls('C3s_sv3', s3(), ['lo','mid','hi'], sv=3)
cls('C3s_utf', s3(), ['é','日本','x'*200])
compile_all()
