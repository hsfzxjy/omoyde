import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

import app.routes
from app.prelude import app
