from typing import Literal, Optional

EventStatus = Literal["Ok", "Info", "Warning", "Critical"]

class FlashDutyClient:
    def __init__(self, integration_key: str) -> None: ...
    def send_alert(
        self,
        event_status: EventStatus,
        title_rule: str,
        alert_key: Optional[str] = None,
        description: Optional[str] = None,
        labels: Optional[dict[str, str]] = None,
        images: Optional[list[dict[str, str]]] = None,
    ) -> None: ...
    def shutdown(self) -> None: ...
