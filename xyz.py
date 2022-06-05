def from_xyz_lines(cls, lines, iter_from=0, add_comment=""):
  atom_count = int(lines[0])
  comment = lines[1]
  iter_num = int(comment[comment.find('iter:') + 5:].split()[0]) + iter_from
  comment = f"iter:{iter_num} {add_comment}\n"
  atom_params = [line.split() for line in lines[2:]]
  atoms = []
  for atom_param in atom_params:
    atoms.append(
      Atom(
        atom_param[0],
        [float(v) for v in atom_param[1:4]],
        float(atom_param[4]),
        [float(v) for v in atom_param[5:8]]
      )
    )
  return MDFrame(iter_num, atom_count, comment, atoms)

@classmethod
def from_xyz(cls, filepath, iter_from=0):
  """load MD frames from .xyz file"""
  frames = []
  with open(filepath, "r") as f:
    while True:
      lines = []
      header = f.readline()
      if header.strip() == '':
        break
      try:
        atom_count = int(header)
      except ValueError as e:
        raise ValueError('Expected xyz header but got: {}'.format(e))
      lines.append(atom_count)
      lines.append(f.readline())
      for _i in range(atom_count):
        lines.append(f.readline())
      frames.append(
        MDFrame.from_xyz_lines(lines, iter_from=iter_from)
      )
  return MDTrajectory(frames)
