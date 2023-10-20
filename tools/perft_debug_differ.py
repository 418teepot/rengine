a = """b2b3: 1
g2g3: 1
d5d6: 1
g2g4: 1
g2h3: 1
d5e6: 1
b5c6: 1
d5c6: 1
c3b1: 1
c3d1: 1
c3a2: 1
c3a4: 1
e5d3: 1
e5c4: 1
e5g4: 1
e5c6: 1
e5g6: 1
e5d7: 1
e5f7: 1
d2c1: 1
d2e3: 1
d2f4: 1
d2g5: 1
d2h6: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2c4: 1
a1b1: 1
a1c1: 1
a1d1: 1
a1a2: 1
a1a3: 1
a1a4: 1
a1a5: 1
a1a6: 1
a1a7: 1
h1f1: 1
h1g1: 1
f3d3: 1
f3e3: 1
f3g3: 1
f3h3: 1
f3f4: 1
f3g4: 1
f3f5: 1
f3h5: 1
f3f6: 1
e1d1: 1
e1f1: 1
e1g1: 1
e1c1: 1"""

b = """e1d1: 1
e1f1: 1
c3b1: 1
c3d1: 1
c3a2: 1
c3a4: 1
e5g6: 1
e5d7: 1
e5f7: 1
e5d3: 1
e5c4: 1
e5g4: 1
e5c6: 1
a1a7: 1
a1b1: 1
a1c1: 1
a1d1: 1
a1a2: 1
a1a3: 1
a1a4: 1
a1a5: 1
a1a6: 1
h1f1: 1
h1g1: 1
d2c1: 1
d2e3: 1
d2f4: 1
d2g5: 1
d2h6: 1
e2d1: 1
e2f1: 1
e2d3: 1
e2c4: 1
f3h3: 1
f3f6: 1
f3d3: 1
f3e3: 1
f3g3: 1
f3f4: 1
f3f5: 1
f3g4: 1
f3h5: 1
b2b3: 1
g2g3: 1
d5d6: 1
g2g4: 1
g2h3: 1
d5e6: 1
d5c6: 1
e1c1: 1
e1g1: 1"""

def list_and_sort(a: str) -> list[str]:
    l = [new.strip() for new in a.splitlines()]
    l.sort()
    return l

def compare_list(a: list[str], b: list[str]):
    temp = []
    for elem in a:
        if elem not in b:
            temp.append(elem)
    for t in temp:
        print(f'{t}')

if __name__ == '__main__':
    lista = list_and_sort(a)
    listb = list_and_sort(b)
    compare_list(lista, listb)