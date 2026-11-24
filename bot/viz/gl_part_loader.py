import open3d as o3d
from gltflib import GLTF
import numpy as np


def get_byte_buffer(gltf_scenes, buffer_view_index):
    gltf_model = gltf_scenes.model
    buffer_view = gltf_model.bufferViews[buffer_view_index]
    buffer_index = buffer_view.buffer
    buffer_offset = buffer_view.byteOffset
    buffer_length = buffer_view.byteLength
    resource = gltf_scenes.resources[buffer_index]
    raw_buffer_view = memoryview(resource.data)[
        buffer_offset : buffer_offset + buffer_length
    ].tobytes()
    return raw_buffer_view


def get_vertex_buffer(gltf_scenes, attributes_struct):
    position_data_index = attributes_struct.POSITION
    vertices_buffer = get_byte_buffer(gltf_scenes, position_data_index)
    vertices = np.frombuffer(vertices_buffer, dtype=np.float32)
    vertices = vertices.reshape((-1, 3))
    return o3d.utility.Vector3dVector(vertices)


def get_index_buffer(gltf_scenes, indices_data_index):
    indices_buffer = get_byte_buffer(gltf_scenes, indices_data_index)
    indices = np.frombuffer(indices_buffer, dtype=np.int16)
    indices = indices.reshape((-1, 3))
    return o3d.utility.Vector3iVector(indices)


def get_normal_buffer(gltf_scenes, attributes_struct):
    position_data_index = attributes_struct.NORMAL
    vertices_buffer = get_byte_buffer(gltf_scenes, position_data_index)
    vertices = np.frombuffer(vertices_buffer, dtype=np.float32)
    vertices = vertices.reshape((-1, 3))
    return o3d.utility.Vector3dVector(vertices)
