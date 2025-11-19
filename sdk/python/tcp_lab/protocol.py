from abc import ABC, abstractmethod
from typing import Protocol
from .structs import Packet

class SystemContext(Protocol):
    def send_packet(self, packet: Packet) -> None:
        ...
        
    def start_timer(self, delay_ms: int, timer_id: int) -> None:
        ...
        
    def cancel_timer(self, timer_id: int) -> None:
        ...
        
    def deliver_data(self, data: bytes) -> None:
        ...
        
    def log(self, message: str) -> None:
        ...
        
    def now(self) -> int:
        ...
        
    def record_metric(self, name: str, value: float) -> None:
        ...

class TransportProtocol(ABC):
    def init(self, ctx: SystemContext) -> None:
        pass
        
    @abstractmethod
    def on_packet(self, ctx: SystemContext, packet: Packet) -> None:
        pass
        
    @abstractmethod
    def on_timer(self, ctx: SystemContext, timer_id: int) -> None:
        pass
        
    @abstractmethod
    def on_app_data(self, ctx: SystemContext, data: bytes) -> None:
        pass

