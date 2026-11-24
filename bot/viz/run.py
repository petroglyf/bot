import numpy as np
import open3d as o3d
from gltflib import GLTF

from bot.viz.gl_part_loader import get_index_buffer, get_vertex_buffer

# print("Testing IO for meshes ...")
# mesh = o3d.io.read_triangle_mesh("nao.glb", print_progress=True)
# print(mesh)


class dfs_node:
  def __init__(self, name, translation, mesh):
    self.name = name
    self.translation = translation
    print("Self translation: " + str(self.translation))
    self.mesh = mesh
    self.children = []

  def __repr__(self):
    return f"Node(name={self.name}, translation={self.translation})"


def o3d_mesh_from_gltf(loaded_gltf, gltf_mesh) -> o3d.geometry.TriangleMesh:
  mesh_out = o3d.geometry.TriangleMesh()
  mesh_out.vertices = get_vertex_buffer(loaded_gltf, gltf_mesh.primitives[0].attributes)
  mesh_out.triangles = get_index_buffer(loaded_gltf, gltf_mesh.primitives[0].indices)
  # mesh_out.vertex_normals = get_normal_buffer(
  #     gltf_out, mesh.primitives[0].attributes
  # )
  mesh_out.compute_vertex_normals()
  # mesh.triangles = o3d.utility.Vector3iVector(np.arange(28215).reshape((-1, 3)))
  return mesh_out


gltf_out = GLTF.load("nao.glb")
gltf_model = gltf_out.model
current_scene = gltf_model.scenes[gltf_model.scene]

mesh_lib = []
for mesh in gltf_model.meshes:
  print(f"Mesh with name: {mesh.name}")
  mesh_out = o3d_mesh_from_gltf(gltf_out, mesh)
  mesh_lib.append(mesh_out)


node_lib = []
# vis = o3d.visualization.Visualizer()
# vis.create_window()

for i, node in enumerate(gltf_model.nodes):
  print(f"{i}: Node with name: {node.name}")
  mesh = mesh_lib[node.mesh] if node.mesh is not None else None
  translation = (
    np.array(node.translation) if node.translation is not None else np.zeros((3))
  )
  print(" adding with translation: " + str(translation))
  node_lib.append(dfs_node(node.name, translation, mesh))


def dfs_add(cur_node_index):
  cur_node_inst = node_lib[cur_node_index]
  cur_node = gltf_model.nodes[cur_node_index]
  if cur_node.children is None:
    return
  for child_index in cur_node.children:
    print("Child index: " + str(child_index))
    child_node = node_lib[child_index]
    print(f" Adding child {child_node.name} to node {cur_node.name}")
    cur_node_inst.children.append(child_node)
    dfs_add(child_index)


root_index = gltf_model.scenes[0].nodes[0]
root = node_lib[root_index]
dfs_add(root_index)

center = np.zeros((3))


def dfs_translate(cur_node, translation_base):
  print("Base: " + str(translation_base))
  print("cn_trans: " + str(cur_node.translation))
  next_translation = translation_base + cur_node.translation

  if cur_node.mesh is not None:
    cur_node.mesh.translate(next_translation)

  for child_node in cur_node.children:
    print("translating: " + str(child_node.name))
    dfs_translate(child_node, next_translation)


dfs_translate(root, center)
# mesh.transform(transform)
# o3d.visualization.draw_geometries([mesh])

# vis.add_geometry(list(mesh_lib.values()))
# vis.run()

# vis.destroy_window()
# o3d.visualization.draw_geometries(list(node_lib.values()))
# o3d.visualization.draw(list(mesh_lib.values()))


# mesh = mesh_lib[0]
# mesh.translate(np.array([0, 0.1, 0]))
o3d.visualization.draw_geometries(
  mesh_lib,
  lookat=[0, 0, 0],
  up=[0, 1, 0],
  front=[0, 0, -1],
  zoom=0.5,
)
