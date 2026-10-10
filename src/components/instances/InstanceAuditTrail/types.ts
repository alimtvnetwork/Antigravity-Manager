export interface InstanceAuditTrailModalProps {
    instance: {
        id: string;
        name: string;
        sequence_name?: string;
    } | null;
    isOpen: boolean;
    onClose: () => void;
}
