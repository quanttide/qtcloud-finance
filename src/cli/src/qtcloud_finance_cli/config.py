"""配置管理模块"""

from pathlib import Path

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """统一配置"""

    llm_model: str = "deepseek-v4-flash"
    llm_base_url: str = "https://api.deepseek.com"
    llm_api_key: str = ""

    data_root: Path = Path(__file__).parent.parent.parent / "data"

    temperature: float = 0.1
    max_tokens: int = 1000

    class Config:
        env_prefix = ""
        extra = "ignore"
