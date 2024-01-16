

use bevy::{
    prelude::*, 
    utils::{HashMap, HashSet}, tasks::Task
};
use bevy_xpbd_3d::components::Collider;

use super::{chunk::*, TerrainMaterial};



use std::sync::{Arc, RwLock};


// pub enum ChunkMeshingState {
//     Pending,
//     Meshing,//(Task<Mesh>),
//     Completed,
// }

// Box<Chunk>;         not supported for SharedPtr
// Arc<RwLock<Chunk>>; non convinent for readonly ops
pub type ChunkPtr = Arc<RwLock<Chunk>>;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ChunkSystem {

    /// all loaded chunks.
    /// ChunkList can be read (by multiple threads) at the same time, but only one can be writing at the same time and no other can be reading at this time.
    // 设计一个高性能区块系统，这两个区块列表 及每个区块 都有RwLock特性，即 可同时可被多处读，但只能被互斥写
    // linear-list of loaded chunks.
    // chunks: Arc<RwLock<HashMap<IVec3, Arc<RwLock<Chunk>>>>>, 
    #[reflect(ignore)]
    pub chunks: HashMap<IVec3, ChunkPtr>,

    // Spare Voxel Octree for Spatial lookup acceleration.
    // chunks_svo: SVO<Arc<RwLock<Chunk>>>,

    // pub chunks_loading: HashSet<IVec3>,
    pub chunks_remesh: HashSet<IVec3>,      // marked as ReMesh
    
    #[reflect(ignore)]
    pub chunks_meshing: HashMap<IVec3, Task<(Mesh, Option<Collider>, Entity, Handle<Mesh>)>>,

    pub view_distance: IVec2,

    pub entity: Entity,

    pub vox_mtl: Handle<TerrainMaterial>,

    pub dbg_remesh_all_chunks: bool,

}

impl Default for ChunkSystem {
    fn default() -> Self {
        Self {
            chunks: HashMap::default(),//Arc::new(RwLock::new(HashMap::new())), 
            view_distance: IVec2::new(1, 1),
            entity: Entity::PLACEHOLDER,
            vox_mtl: Handle::default(),
            dbg_remesh_all_chunks: false,
            chunks_remesh: HashSet::default(),
            chunks_meshing: HashMap::default(),
        }
    }
}

impl ChunkSystem {

    pub fn new(view_distance: i32) -> Self {
        Self { 
            view_distance: IVec2::new(view_distance, view_distance),
            // chunks_loading: HashSet::new(),
            // chunks_meshing: HashMap::new(),
            ..default()
        }
    }

    pub fn get_chunk(&self, chunkpos: IVec3) -> Option<&ChunkPtr> {
        assert!(Chunk::is_chunkpos(chunkpos));

        if let Some(chunk) = self.chunks.get(&chunkpos) {  //.read().unwrap().get(&chunkpos) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn has_chunk(&self, chunkpos: IVec3) -> bool {
        assert!(Chunk::is_chunkpos(chunkpos));

         self.chunks.contains_key(&chunkpos)  //.read().unwrap().contains_key(&chunkpos)
    }

    pub fn num_chunks(&self) -> usize {

        self.chunks.len() //.read().unwrap().len()
    }

    // pub fn provide_chunk(&self, chunkpos: IVec3) -> ChunkPtr {
    //     assert!(!self.has_chunk(chunkpos));

    //     let mut chunk = Arc::new(RwLock::new(Chunk::new(chunkpos)));

    //     let load = false;  // chunk_loader.load_chunk(chunk);

    //     if !load {

    //         ChunkGenerator::generate_chunk(chunk.write().unwrap().borrow_mut());
    //     }

    //     chunk
    // }


    pub fn spawn_chunk(&mut self, chunkptr: ChunkPtr) {
        let chunkpos;
        {
            let mut chunk = chunkptr.write().unwrap();
            chunkpos = chunk.chunkpos;
    
            for neib_idx in 0..Chunk::NEIGHBOR_DIR.len() {
                let neib_dir = Chunk::NEIGHBOR_DIR[neib_idx];
                let neib_chunkpos = chunkpos + neib_dir * Chunk::SIZE;
    
                self.mark_chunk_remesh(neib_chunkpos);

                // set neighbor_chunks cache
                chunk.neighbor_chunks[neib_idx] = 
                if let Some(neib_chunkptr) = self.get_chunk(neib_chunkpos) {

                    // update neighbor's `neighbor_chunk`
                    neib_chunkptr.write().unwrap().neighbor_chunks[Chunk::neighbor_idx_opposite(neib_idx)] = Some(Arc::downgrade(&chunkptr));

                    Some(Arc::downgrade(neib_chunkptr))
                } else {
                    None
                }
            }

        }

        self.chunks.insert(chunkpos, chunkptr); 
        
        // // There is no need to cast shadows for chunks below the surface.
        // if chunkpos.y <= 64 {
        //     entity_commands.insert(NotShadowCaster);
        // }

        // self.set_chunk_meshing(chunkpos, ChunkMeshingState::Pending);

    }

    pub fn despawn_chunk(&mut self, chunkpos: IVec3) -> Option<ChunkPtr> {

        if let Some(chunk) = self.chunks.remove(&chunkpos) {  //.write().unwrap()

            //cmds.entity(chunk.entity).despawn_recursive();

            Some(chunk)
        } else {
            None
        }
    }

    pub fn mark_chunk_remesh(&mut self, chunkpos: IVec3) {
        self.chunks_remesh.insert(chunkpos);
    }

    // pub fn set_chunk_meshing(&mut self, chunkpos: IVec3, stat: ChunkMeshingState) {
    //     self.chunks_meshing.insert(chunkpos, stat);
    // }

}






