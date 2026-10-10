import { useState, useRef } from 'react';
import type { SyncInterval } from '../../../services/instanceService';
import { getPromptTreeSyncInterval } from '../../../services/instanceService';
import { getArchivedProjectsForInstance } from '../../../services/instanceService';
import type { AgmConversationNode, AgmProjectTreeNode, ViewMode } from './types';

export interface InspectorPrompt {
    title: string;
    text: string;
    convId: string;
    projectPath: string;
    images: string[];
    files: string[];
}

export interface PromptTreeState {
    treeData: AgmProjectTreeNode[];
    setTreeData: React.Dispatch<React.SetStateAction<AgmProjectTreeNode[]>>;
    isLoading: boolean;
    setIsLoading: React.Dispatch<React.SetStateAction<boolean>>;
    isAutoSyncing: boolean;
    setIsAutoSyncing: React.Dispatch<React.SetStateAction<boolean>>;
    syncInterval: SyncInterval;
    setSyncInterval: React.Dispatch<React.SetStateAction<SyncInterval>>;
    error: string | null;
    setError: React.Dispatch<React.SetStateAction<string | null>>;
    searchQuery: string;
    setSearchQuery: React.Dispatch<React.SetStateAction<string>>;
    activeFilter: 'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived';
    setActiveFilter: React.Dispatch<React.SetStateAction<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>>;
    categoryFilter: 'all' | 'user' | 'subagent' | 'system';
    setCategoryFilter: React.Dispatch<React.SetStateAction<'all' | 'user' | 'subagent' | 'system'>>;
    restoreFileInputRef: React.RefObject<HTMLInputElement | null>;
    isFullscreen: boolean;
    setIsFullscreen: React.Dispatch<React.SetStateAction<boolean>>;
    pinnedProjectIds: string[];
    setPinnedProjectIds: React.Dispatch<React.SetStateAction<string[]>>;
    archivedProjectIds: string[];
    setArchivedProjectIds: React.Dispatch<React.SetStateAction<string[]>>;
    isArchivedProjectsExpanded: boolean;
    setIsArchivedProjectsExpanded: React.Dispatch<React.SetStateAction<boolean>>;
    refreshingProjectId: string | null;
    setRefreshingProjectId: React.Dispatch<React.SetStateAction<string | null>>;
    confirmationSuffix: string;
    setConfirmationSuffix: React.Dispatch<React.SetStateAction<string>>;
    isCopiedText: boolean;
    setIsCopiedText: React.Dispatch<React.SetStateAction<boolean>>;
    isCopiedRaw: boolean;
    setIsCopiedRaw: React.Dispatch<React.SetStateAction<boolean>>;
    selectedProjectScope: string;
    setSelectedProjectScope: React.Dispatch<React.SetStateAction<string>>;
    isDetailsModalOpen: boolean;
    setIsDetailsModalOpen: React.Dispatch<React.SetStateAction<boolean>>;
    isFocusing: boolean;
    setIsFocusing: React.Dispatch<React.SetStateAction<boolean>>;
    copiedField: string | null;
    setCopiedField: React.Dispatch<React.SetStateAction<string | null>>;
    expandedProjects: Record<string, boolean>;
    setExpandedProjects: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    expandedStaleGroups: Record<string, boolean>;
    setExpandedStaleGroups: React.Dispatch<React.SetStateAction<Record<string, boolean>>>;
    selectedProject: AgmProjectTreeNode | null;
    setSelectedProject: React.Dispatch<React.SetStateAction<AgmProjectTreeNode | null>>;
    selectedConversation: AgmConversationNode | null;
    setSelectedConversation: React.Dispatch<React.SetStateAction<AgmConversationNode | null>>;
    activePromptText: string;
    setActivePromptText: React.Dispatch<React.SetStateAction<string>>;
    editedPromptText: string;
    setEditedPromptText: React.Dispatch<React.SetStateAction<string>>;
    viewMode: ViewMode;
    setViewMode: React.Dispatch<React.SetStateAction<ViewMode>>;
    showAllWords: boolean;
    setShowAllWords: React.Dispatch<React.SetStateAction<boolean>>;
    previewTab: 'instruction' | 'results';
    setPreviewTab: React.Dispatch<React.SetStateAction<'instruction' | 'results'>>;
    isCopiedResults: boolean;
    setIsCopiedResults: React.Dispatch<React.SetStateAction<boolean>>;
    isCopied: boolean;
    setIsCopied: React.Dispatch<React.SetStateAction<boolean>>;
    isResending: boolean;
    setIsResending: React.Dispatch<React.SetStateAction<boolean>>;
    isEnqueueing: boolean;
    setIsEnqueueing: React.Dispatch<React.SetStateAction<boolean>>;
    isExportMenuOpen: boolean;
    setIsExportMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
    actionMsg: string | null;
    setActionMsg: React.Dispatch<React.SetStateAction<string | null>>;
    selectedConversationRef: React.MutableRefObject<AgmConversationNode | null>;
    selectedProjectRef: React.MutableRefObject<AgmProjectTreeNode | null>;
    activePromptTextRef: React.MutableRefObject<string>;
    elapsedSeconds: number;
    setElapsedSeconds: React.Dispatch<React.SetStateAction<number>>;
    inspectorPrompt: InspectorPrompt | null;
    setInspectorPrompt: React.Dispatch<React.SetStateAction<InspectorPrompt | null>>;
}

export function usePromptTreeState(instanceId: string): PromptTreeState {
    const [treeData, setTreeData] = useState<AgmProjectTreeNode[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [isAutoSyncing, setIsAutoSyncing] = useState(false);
    const [syncInterval, setSyncInterval] = useState<SyncInterval>(() => getPromptTreeSyncInterval());
    const [error, setError] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [activeFilter, setActiveFilter] = useState<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>('all');
    const [categoryFilter, setCategoryFilter] = useState<'all' | 'user' | 'subagent' | 'system'>('all');
    const restoreFileInputRef = useRef<HTMLInputElement>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);

    // Pinned projects persisted in localStorage
    const [pinnedProjectIds, setPinnedProjectIds] = useState<string[]>(() => {
        try {
            const stored = localStorage.getItem(`agm_pinned_projects_${instanceId || 'default'}`);
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    });

    // Archived projects persisted in localStorage
    const [archivedProjectIds, setArchivedProjectIds] = useState<string[]>(() => {
        return getArchivedProjectsForInstance(instanceId);
    });

    const [isArchivedProjectsExpanded, setIsArchivedProjectsExpanded] = useState(false);

    const [refreshingProjectId, setRefreshingProjectId] = useState<string | null>(null);
    const [confirmationSuffix, setConfirmationSuffix] = useState<string>('None (Send as is)');
    const [isCopiedText, setIsCopiedText] = useState(false);
    const [isCopiedRaw, setIsCopiedRaw] = useState(false);

    const [selectedProjectScope, setSelectedProjectScope] = useState<string>('all');
    const [isDetailsModalOpen, setIsDetailsModalOpen] = useState(false);
    const [isFocusing, setIsFocusing] = useState(false);
    const [copiedField, setCopiedField] = useState<string | null>(null);

    const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({});
    const [expandedStaleGroups, setExpandedStaleGroups] = useState<Record<string, boolean>>({});

    const [selectedProject, setSelectedProject] = useState<AgmProjectTreeNode | null>(null);
    const [selectedConversation, setSelectedConversation] = useState<AgmConversationNode | null>(null);
    const [activePromptText, setActivePromptText] = useState<string>('');
    const [editedPromptText, setEditedPromptText] = useState<string>('');
    const [viewMode, setViewMode] = useState<ViewMode>('preview');
    const [showAllWords, setShowAllWords] = useState<boolean>(false);
    const [previewTab, setPreviewTab] = useState<'instruction' | 'results'>('instruction');
    const [isCopiedResults, setIsCopiedResults] = useState(false);

    const [isCopied, setIsCopied] = useState(false);
    const [isResending, setIsResending] = useState(false);
    const [isEnqueueing, setIsEnqueueing] = useState(false);
    const [isExportMenuOpen, setIsExportMenuOpen] = useState(false);
    const [actionMsg, setActionMsg] = useState<string | null>(null);

    // Live refs to prevent closure staleness in background auto-sync timer
    const selectedConversationRef = useRef(selectedConversation);
    selectedConversationRef.current = selectedConversation;
    const selectedProjectRef = useRef(selectedProject);
    selectedProjectRef.current = selectedProject;
    const activePromptTextRef = useRef(activePromptText);
    activePromptTextRef.current = activePromptText;

    const [elapsedSeconds, setElapsedSeconds] = useState(0);

    // Full screen prompt detail inspector
    const [inspectorPrompt, setInspectorPrompt] = useState<InspectorPrompt | null>(null);

    return {
        treeData, setTreeData,
        isLoading, setIsLoading,
        isAutoSyncing, setIsAutoSyncing,
        syncInterval, setSyncInterval,
        error, setError,
        searchQuery, setSearchQuery,
        activeFilter, setActiveFilter,
        categoryFilter, setCategoryFilter,
        restoreFileInputRef,
        isFullscreen, setIsFullscreen,
        pinnedProjectIds, setPinnedProjectIds,
        archivedProjectIds, setArchivedProjectIds,
        isArchivedProjectsExpanded, setIsArchivedProjectsExpanded,
        refreshingProjectId, setRefreshingProjectId,
        confirmationSuffix, setConfirmationSuffix,
        isCopiedText, setIsCopiedText,
        isCopiedRaw, setIsCopiedRaw,
        selectedProjectScope, setSelectedProjectScope,
        isDetailsModalOpen, setIsDetailsModalOpen,
        isFocusing, setIsFocusing,
        copiedField, setCopiedField,
        expandedProjects, setExpandedProjects,
        expandedStaleGroups, setExpandedStaleGroups,
        selectedProject, setSelectedProject,
        selectedConversation, setSelectedConversation,
        activePromptText, setActivePromptText,
        editedPromptText, setEditedPromptText,
        viewMode, setViewMode,
        showAllWords, setShowAllWords,
        previewTab, setPreviewTab,
        isCopiedResults, setIsCopiedResults,
        isCopied, setIsCopied,
        isResending, setIsResending,
        isEnqueueing, setIsEnqueueing,
        isExportMenuOpen, setIsExportMenuOpen,
        actionMsg, setActionMsg,
        selectedConversationRef,
        selectedProjectRef,
        activePromptTextRef,
        elapsedSeconds, setElapsedSeconds,
        inspectorPrompt, setInspectorPrompt,
    };
}
