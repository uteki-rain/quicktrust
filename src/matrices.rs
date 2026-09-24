const BM_SIDE: usize = 32;
type BlkMat = [f32; BM_SIDE * BM_SIDE];
type BlkR<'a> = (&'a BlkMat, bool);
type BlkW<'a> = (&'a mut BlkMat, bool);

#[inline]
fn blkclr(out: BlkW) {
    for i in 0..(BM_SIDE * BM_SIDE) { out.0[i] = 0.0; }
}

#[inline]
fn blkmul(lhs: BlkR, rhs: BlkR, out: BlkW) {
    let (lr, ls) = if lhs.1 { (1, BM_SIDE) } else { (BM_SIDE, 1) };
    let (rr, rs) = if rhs.1 { (1, BM_SIDE) } else { (BM_SIDE, 1) };
    let (or, os) = if out.1 { (1, BM_SIDE) } else { (BM_SIDE, 1) };
    for i in 0..BM_SIDE {  for k in 0..BM_SIDE {
        out.0[i * or + k * os] += (0..BM_SIDE)
            .map(|j| lhs.0[i * lr + j * ls] * rhs.0[j * rr + k * rs])
            .sum::<f32>();
    } }
}

#[inline]
fn blkadd(arg: BlkR, scl: f32, out: BlkW) {
    let (gr, gs) = if arg.1 { (1, BM_SIDE) } else { (BM_SIDE, 1) };
    let (or, os) = if out.1 { (1, BM_SIDE) } else { (BM_SIDE, 1) };
    for i in 0..BM_SIDE {  for j in 0..BM_SIDE {
        out.0[i * or + j * os] += scl * arg.0[i * gr + j * gs];
    } }
}

#[inline]
fn blkscl(scl: f32, des: BlkW) {
    for i in 0..(BM_SIDE * BM_SIDE) { des.0[i] *= scl; }
}

#[derive(Clone, Debug)]
pub struct BlockedMatrix {
    blocks: Box<[BlkMat]>,
    bshape: [usize; 2],
    shape: [usize; 2],
}

impl BlockedMatrix {
    pub fn new_zero(shape: [usize; 2]) -> Self {
        let bshape = [
            shape[0].saturating_add(BM_SIDE - 1) / BM_SIDE,
            shape[1].saturating_add(BM_SIDE - 1) / BM_SIDE,
        ];
        Self {
            blocks: vec![[0.0; BM_SIDE * BM_SIDE]; bshape[0] * bshape[1]].into(),
            bshape, shape,
        }
    }
    pub fn clear(&mut self) {
        for b in self.blocks.iter_mut() { blkclr((b, false)); }
    }
    pub fn scale(&mut self, scl: f32) {
        for b in self.blocks.iter_mut() { blkscl(scl, (b, false)); }
    }
    pub fn shape(&self) -> [usize; 2] { self.shape.clone() }
}

impl<'i> std::ops::Index<&'i [usize; 2]> for BlockedMatrix {
    type Output = f32;
    fn index<'a>(&'a self, ij: &'i [usize; 2]) -> &'a f32 {
        let (i, r) = (ij[0] / BM_SIDE, ij[0] % BM_SIDE);
        let (j, c) = (ij[1] / BM_SIDE, ij[1] % BM_SIDE);
        &self.blocks[i * self.bshape[1] + j][r * BM_SIDE + c]
    }
}

impl std::ops::Index<[usize; 2]> for BlockedMatrix {
    type Output = f32;
    fn index(&self, ij: [usize; 2]) -> &f32 {
        let (i, r) = (ij[0] / BM_SIDE, ij[0] % BM_SIDE);
        let (j, c) = (ij[1] / BM_SIDE, ij[1] % BM_SIDE);
        &self.blocks[i * self.bshape[1] + j][r * BM_SIDE + c]
    }
}

impl<'i> std::ops::IndexMut<&'i [usize; 2]> for BlockedMatrix {
    fn index_mut<'a>(&'a mut self, ij: &'i [usize; 2]) -> &'a mut f32 {
        let (i, r) = (ij[0] / BM_SIDE, ij[0] % BM_SIDE);
        let (j, c) = (ij[1] / BM_SIDE, ij[1] % BM_SIDE);
        &mut self.blocks[i * self.bshape[1] + j][r * BM_SIDE + c]
    }
}

impl std::ops::IndexMut<[usize; 2]> for BlockedMatrix {
    fn index_mut(&mut self, ij: [usize; 2]) -> &mut f32 {
        let (i, r) = (ij[0] / BM_SIDE, ij[0] % BM_SIDE);
        let (j, c) = (ij[1] / BM_SIDE, ij[1] % BM_SIDE);
        &mut self.blocks[i * self.bshape[1] + j][r * BM_SIDE + c]
    }
}

pub fn matmul(
    lhs: (&BlockedMatrix, bool),
    rhs: (&BlockedMatrix, bool),
    out: (&mut BlockedMatrix, bool),
) {
    let (lr, ls, la) = if lhs.1 { (1, lhs.0.bshape[1], 1) } else { (lhs.0.bshape[1], 1, 0) };
    let (rr, rs, ra) = if rhs.1 { (1, rhs.0.bshape[1], 1) } else { (rhs.0.bshape[1], 1, 0) };
    let (or, os, oa) = if out.1 { (1, out.0.bshape[1], 1) } else { (out.0.bshape[1], 1, 0) };
    assert_eq!(lhs.0.shape[1 - la], rhs.0.shape[ra]);
    assert_eq!(lhs.0.shape[la], out.0.shape[oa]);
    assert_eq!(rhs.0.shape[1 - ra], out.0.shape[1 - oa]);
    for i in 0..out.0.bshape[oa] {  for k in 0..out.0.bshape[1 - oa] {
        for j in 0..rhs.0.bshape[ra] {
            blkmul(
                (&lhs.0.blocks[i * lr + j * ls], lhs.1),
                (&rhs.0.blocks[j * rr + k * rs], rhs.1),
                (&mut out.0.blocks[i * or + k * os], out.1),
            );
        }
    } }
}

pub fn matadd(
    arg: (&BlockedMatrix, bool),
    scl: f32,
    out: (&mut BlockedMatrix, bool),
) {
    let (gr, gs, ga) = if arg.1 { (1, arg.0.bshape[1], 1) } else { (arg.0.bshape[1], 1, 0) };
    let (or, os, oa) = if out.1 { (1, out.0.bshape[1], 1) } else { (out.0.bshape[1], 1, 0) };
    assert_eq!(arg.0.shape[ga], out.0.shape[oa]);
    assert_eq!(arg.0.shape[1 - ga], out.0.shape[1 - oa]);
    for i in 0..out.0.bshape[oa] {  for j in 0..out.0.bshape[1 - oa] {
        blkadd(
            (&arg.0.blocks[i * gr + j * gs], arg.1),
            scl,
            (&mut out.0.blocks[i * or + j * os], out.1),
        );
    } }
}

pub fn matscl(
    scl: f32,
    out: (&mut BlockedMatrix, bool),
) {
    let (or, os, oa) = if out.1 { (1, out.0.bshape[1], 1) } else { (out.0.bshape[1], 1, 0) };
    for i in 0..out.0.bshape[oa] {  for j in 0..out.0.bshape[1 - oa] {
        blkscl(scl, (&mut out.0.blocks[i * or + j * os], out.1));
    } }
}

pub fn mkmat(shape: [usize; 2], stride: [usize; 2], arr: &[f32]) -> BlockedMatrix {
    let mut m = BlockedMatrix::new_zero(shape.clone());
    for i in 0..shape[0] {  for j in 0..shape[1] {
        m[[i, j]] = arr[i * stride[0] + j * stride[1]];
    } }
    m
}
