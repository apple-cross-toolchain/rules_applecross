import sys,pickle,collections,json; sys.path.insert(0,'/tmp/cgw'); from apex8 import *
T,A,B=pickle.load(open('/tmp/cgw/apex8_5.pkl','rb'))
for e in sys.argv[1:]:
    env=json.loads(e)
    _,M=apex.run(T,env)
    c=collections.Counter()
    for t,a,m in zip(T,A,M):
        A_,P,Q,xp,xq,tp,tq=geom(t)
        c[('same' if (P[0]-A_[0])*(Q[0]-A_[0])>0 else 'opp', a[3][3]==m[3][3])]+=1
    print(env,c)
