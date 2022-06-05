import numpy as np

# Keep in sync with trade-node/src/interface.rs


class NodeInterface:
    def get_data() -> np.array:
        raise NotImplementedError()
