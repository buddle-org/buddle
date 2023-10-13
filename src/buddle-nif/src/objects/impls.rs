use buddle_math::{BVec4, Vec2, Vec3, Vec4};

use super::*;

impl NiObject {
    /// Gets a list of child references stored in this
    /// block, if any.
    pub fn child_refs(&self) -> Option<&[Ref<NiAVObject>]> {
        let ninode = match self {
            NiObject::NiNode(block) => Some(block),
            NiObject::NiBone(block) => Some(&block.base),
            NiObject::NiCollisionSwitch(block) => Some(&block.base),
            NiObject::NiBillboardNode(block) => Some(&block.base),
            NiObject::NiSwitchNode(block) => Some(&block.base),
            NiObject::NiLODNode(block) => Some(&block.base.base),
            NiObject::NiSortAdjustNode(block) => Some(&block.base),
            NiObject::NiRoomGroup(block) => Some(&block.base),
            NiObject::NiWall(block) => Some(&block.base),
            NiObject::NiRoom(block) => Some(&block.base),
            _ => None,
        };

        ninode.map(|n| n.children.as_slice())
    }

    /// Gets a list of [`NiObject`] references to child nodes
    /// referenced by this block, if any.
    pub fn children<'b>(&self, blocks: &'b [NiObject]) -> Option<Vec<&'b NiObject>> {
        self.child_refs()
            .map(|refs| refs.iter().filter_map(|r| r.get(blocks)).collect())
    }

    /// Gets the AVObject part of an object, if it exists
    pub fn avobject(&self) -> Option<&NiAVObject> {
        match self {
            NiObject::NiAVObject(block) => Some(block),
            NiObject::NiDynamicEffect(block) => Some(&block.base),
            NiObject::NiLight(block) => Some(&block.base.base),
            NiObject::NiAmbientLight(block) => Some(&block.base.base.base),
            NiObject::NiDirectionalLight(block) => Some(&block.base.base.base),
            NiObject::NiPointLight(block) => Some(&block.base.base.base),
            NiObject::NiSpotLight(block) => Some(&block.base.base.base.base),
            NiObject::NiTextureEffect(block) => Some(&block.base.base),
            NiObject::NiGeometry(block) => Some(&block.base),
            NiObject::NiTriBasedGeom(block) => Some(&block.base.base),
            NiObject::NiTriShape(block) => Some(&block.base.base.base),
            NiObject::NiScreenElements(block) => Some(&block.base.base.base.base),
            NiObject::NiTriStrips(block) => Some(&block.base.base.base),
            NiObject::NiClod(block) => Some(&block.base.base.base),
            NiObject::NiLines(block) => Some(&block.base.base.base),
            NiObject::NiParticles(block) => Some(&block.base.base),
            NiObject::NiAutoNormalParticles(block) => Some(&block.base.base.base),
            NiObject::NiParticleMeshes(block) => Some(&block.base.base.base),
            NiObject::NiParticleSystem(block) => Some(&block.base.base.base),
            NiObject::NiMeshParticleSystem(block) => Some(&block.base.base.base.base),
            NiObject::NiCamera(block) => Some(&block.base),
            NiObject::NiNode(block) => Some(&block.base),
            NiObject::NiBone(block) => Some(&block.base.base),
            NiObject::NiCollisionSwitch(block) => Some(&block.base.base),
            NiObject::NiBillboardNode(block) => Some(&block.base.base),
            NiObject::NiSwitchNode(block) => Some(&block.base.base),
            NiObject::NiLODNode(block) => Some(&block.base.base.base),
            NiObject::NiSortAdjustNode(block) => Some(&block.base.base),
            NiObject::NiRoomGroup(block) => Some(&block.base.base),
            NiObject::NiWall(block) => Some(&block.base.base),
            NiObject::NiRoom(block) => Some(&block.base.base),
            NiObject::NiBezierMesh(block) => Some(&block.base),
            NiObject::NiPortal(block) => Some(&block.base),
            NiObject::NiRenderObject(block) => Some(&block.base),
            NiObject::NiMesh(block) => Some(&block.base.base),
            NiObject::NiPSParticleSystem(block) => Some(&block.base.base.base),
            NiObject::NiPSMeshParticleSystem(block) => Some(&block.base.base.base.base),
            NiObject::NiMeshHWInstance(block) => Some(&block.base),
            _ => None,
        }
    }

    /// Gets a list of property references stored in this
    /// block, if any.
    pub fn property_refs(&self) -> Option<&[Ref<NiProperty>]> {
        let niavobject = self.avobject();

        niavobject.map(|n| n.properties.as_slice())
    }

    /// Gets a list of [`NiObject`] properties referenced by
    /// this block, if any.
    pub fn properties<'b>(&self, blocks: &'b [NiObject]) -> Option<Vec<&'b NiObject>> {
        self.property_refs()
            .map(|refs| refs.iter().filter_map(|r| r.get(blocks)).collect())
    }

    /// Gets a list of extra data references stored in this
    /// block, if any.
    pub fn extra_data_refs(&self) -> Option<&[Ref<NiExtraData>]> {
        let niobjectnet = match self {
            NiObject::NiObjectNET(block) => Some(block),
            NiObject::NiAVObject(block) => Some(&block.base),
            NiObject::NiDynamicEffect(block) => Some(&block.base.base),
            NiObject::NiLight(block) => Some(&block.base.base.base),
            NiObject::NiAmbientLight(block) => Some(&block.base.base.base.base),
            NiObject::NiDirectionalLight(block) => Some(&block.base.base.base.base),
            NiObject::NiPointLight(block) => Some(&block.base.base.base.base),
            NiObject::NiSpotLight(block) => Some(&block.base.base.base.base.base),
            NiObject::NiTextureEffect(block) => Some(&block.base.base.base),
            NiObject::NiGeometry(block) => Some(&block.base.base),
            NiObject::NiTriBasedGeom(block) => Some(&block.base.base.base),
            NiObject::NiTriShape(block) => Some(&block.base.base.base.base),
            NiObject::NiScreenElements(block) => Some(&block.base.base.base.base.base),
            NiObject::NiTriStrips(block) => Some(&block.base.base.base.base),
            NiObject::NiClod(block) => Some(&block.base.base.base.base),
            NiObject::NiLines(block) => Some(&block.base.base.base.base),
            NiObject::NiParticles(block) => Some(&block.base.base.base),
            NiObject::NiAutoNormalParticles(block) => Some(&block.base.base.base.base),
            NiObject::NiParticleMeshes(block) => Some(&block.base.base.base.base),
            NiObject::NiParticleSystem(block) => Some(&block.base.base.base.base),
            NiObject::NiMeshParticleSystem(block) => Some(&block.base.base.base.base.base),
            NiObject::NiCamera(block) => Some(&block.base.base),
            NiObject::NiNode(block) => Some(&block.base.base),
            NiObject::NiBone(block) => Some(&block.base.base.base),
            NiObject::NiCollisionSwitch(block) => Some(&block.base.base.base),
            NiObject::NiBillboardNode(block) => Some(&block.base.base.base),
            NiObject::NiSwitchNode(block) => Some(&block.base.base.base),
            NiObject::NiLODNode(block) => Some(&block.base.base.base.base),
            NiObject::NiSortAdjustNode(block) => Some(&block.base.base.base),
            NiObject::NiRoomGroup(block) => Some(&block.base.base.base),
            NiObject::NiWall(block) => Some(&block.base.base.base),
            NiObject::NiRoom(block) => Some(&block.base.base.base),
            NiObject::NiBezierMesh(block) => Some(&block.base.base),
            NiObject::NiPortal(block) => Some(&block.base.base),
            NiObject::NiRenderObject(block) => Some(&block.base.base),
            NiObject::NiMesh(block) => Some(&block.base.base.base),
            NiObject::NiPSParticleSystem(block) => Some(&block.base.base.base.base),
            NiObject::NiPSMeshParticleSystem(block) => Some(&block.base.base.base.base.base),
            NiObject::NiMeshHWInstance(block) => Some(&block.base.base),
            NiObject::NiProperty(block) => Some(&block.base),
            NiObject::NiTransparentProperty(block) => Some(&block.base.base),
            NiObject::NiAlphaProperty(block) => Some(&block.base.base),
            NiObject::NiDitherProperty(block) => Some(&block.base.base),
            NiObject::NiFogProperty(block) => Some(&block.base.base),
            NiObject::NiMaterialProperty(block) => Some(&block.base.base),
            NiObject::NiShadeProperty(block) => Some(&block.base.base),
            NiObject::NiSpecularProperty(block) => Some(&block.base.base),
            NiObject::NiStencilProperty(block) => Some(&block.base.base),
            NiObject::NiTextureModeProperty(block) => Some(&block.base.base),
            NiObject::NiTextureProperty(block) => Some(&block.base.base),
            NiObject::NiTexturingProperty(block) => Some(&block.base.base),
            NiObject::NiMultiTextureProperty(block) => Some(&block.base.base.base),
            NiObject::NiVertexColorProperty(block) => Some(&block.base.base),
            NiObject::NiWireframeProperty(block) => Some(&block.base.base),
            NiObject::NiZBufferProperty(block) => Some(&block.base.base),
            NiObject::NiSequenceStreamHelper(block) => Some(&block.base),
            NiObject::NiTexture(block) => Some(&block.base),
            NiObject::NiSourceTexture(block) => Some(&block.base.base),
            NiObject::NiSourceCubeMap(block) => Some(&block.base.base.base),
            NiObject::NiEnvMappedTriShape(block) => Some(&block.base),
            _ => None,
        };

        niobjectnet.map(|n| n.extra_data_list.as_slice())
    }

    /// Resolves the [`NiObject`] extra data referenced by this
    /// block, if any.
    pub fn extra_data<'b>(&self, blocks: &'b [NiObject]) -> Option<Vec<&'b NiObject>> {
        self.extra_data_refs()
            .map(|refs| refs.iter().filter_map(|r| r.get(blocks)).collect())
    }
}

impl NiDataStream {
    pub fn read_with<P, T: Copy, F: Fn(&mut &[P]) -> T>(&self, reader: F) -> Vec<Vec<T>> {
        let slice = self.data.as_slice();
        let mut stream =
            unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const P, slice.len()) };
        let mut res = Vec::with_capacity(self.regions.len());
        for region in &self.regions {
            let mut reg = Vec::with_capacity(region.num_indices as usize);

            for _ in 0..region.num_indices {
                reg.push(reader(&mut stream));
            }

            res.push(reg);
        }

        res
    }

    pub fn read_primitive<T: Copy>(&self) -> Vec<Vec<T>> {
        self.read_with(|prims| *prims.take_first().unwrap())
    }

    pub fn read_vec2(&self) -> Vec<Vec<Vec2>> {
        self.read_with(|floats| {
            let x = *floats.take_first().unwrap();
            let y = *floats.take_first().unwrap();
            Vec2::new(x, y)
        })
    }

    pub fn read_vec3(&self) -> Vec<Vec<Vec3>> {
        self.read_with(|floats| {
            let x = *floats.take_first().unwrap();
            let y = *floats.take_first().unwrap();
            let z = *floats.take_first().unwrap();
            Vec3::new(x, y, z)
        })
    }

    pub fn read_color4(&self) -> Vec<Vec<Vec4>> {
        self.read_with(|bytes| {
            let b: u8 = *bytes.take_first().unwrap();
            let g: u8 = *bytes.take_first().unwrap();
            let r: u8 = *bytes.take_first().unwrap();
            let a: u8 = *bytes.take_first().unwrap();
            Vec4::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            )
        })
    }
}
