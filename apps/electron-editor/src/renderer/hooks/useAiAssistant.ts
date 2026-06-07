import { useState, useCallback } from 'react';
import { useSettingsStore } from '../store/settingsStore';
import { ScenarioStep } from '../types';
import { STEP_SCHEMAS } from '../types/stepSchema';

export interface AiSuggestion {
  name: string;
  type: string;
  description: string;
  data: Record<string, any>;
}

interface AiAssistantState {
  loading: boolean;
  error: string | null;
  suggestions: AiSuggestion[];
  lastQuery: string;
}

export const useAiAssistant = () => {
  const [state, setState] = useState<AiAssistantState>({
    loading: false,
    error: null,
    suggestions: [],
    lastQuery: '',
  });

  const { apiKeyAnthropic } = useSettingsStore();

  /**
   * Validate that a step type exists in STEP_SCHEMAS
   */
  const validateStepType = useCallback((stepType: string): boolean => {
    return stepType in STEP_SCHEMAS;
  }, []);

  /**
   * Generate steps based on user description using Claude API
   */
  const generateSteps = useCallback(
    async (userDescription: string): Promise<AiSuggestion[]> => {
      if (!apiKeyAnthropic) {
        const error = 'Anthropic API key not configured';
        setState((prev) => ({ ...prev, error }));
        throw new Error(error);
      }

      if (!userDescription.trim()) {
        const error = 'Please describe what you want to do';
        setState((prev) => ({ ...prev, error }));
        throw new Error(error);
      }

      setState((prev) => ({
        ...prev,
        loading: true,
        error: null,
        lastQuery: userDescription,
      }));

      try {
        const systemPrompt = `You are an RPA automation expert. Your task is to generate a list of YAML-compatible step definitions for Robost automation scenarios.

Available step types: ${Object.keys(STEP_SCHEMAS).join(', ')}

For each step type, understand these schemas:
${Object.entries(STEP_SCHEMAS)
  .map(
    ([key, schema]) =>
      `- ${key}: ${schema.fields.map((f) => `${f.name}${f.required ? ' (required)' : ''}`).join(', ')}`
  )
  .join('\n')}

Generate realistic automation steps that match user requests. Respond with a JSON array only (no markdown, no explanations).
Each step must have: name (friendly name), type (from available types), description (what it does), and data (field values).`;

        const response = await fetch('https://api.anthropic.com/v1/messages', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'x-api-key': apiKeyAnthropic,
            'anthropic-version': '2023-06-01',
          },
          body: JSON.stringify({
            model: 'claude-opus-4-1-20250805',
            max_tokens: 2048,
            system: systemPrompt,
            messages: [
              {
                role: 'user',
                content: `Generate automation steps for: ${userDescription}`,
              },
            ],
          }),
        });

        if (!response.ok) {
          const errorData = await response.json();
          const errorMsg =
            errorData.error?.message ||
            `API Error: ${response.status} ${response.statusText}`;
          throw new Error(errorMsg);
        }

        const data = await response.json();
        const contentBlock = data.content[0];
        const responseText = contentBlock.type === 'text' ? contentBlock.text : '';

        // Parse JSON from response
        let suggestions: AiSuggestion[] = [];
        try {
          // Extract JSON array from response (might be wrapped in markdown code blocks)
          const jsonMatch =
            responseText.match(/\[[\s\S]*\]/) ||
            responseText.match(/```json\s*([\s\S]*?)\s*```/);
          const jsonString = jsonMatch ? jsonMatch[1] || jsonMatch[0] : responseText;
          suggestions = JSON.parse(jsonString);
        } catch (parseErr) {
          throw new Error('Failed to parse AI response as JSON');
        }

        // Validate step types and filter out invalid ones
        const validSuggestions = suggestions.filter((sugg) => {
          if (!validateStepType(sugg.type)) {
            console.warn(`Invalid step type: ${sugg.type}`);
            return false;
          }
          return true;
        });

        if (validSuggestions.length === 0) {
          throw new Error('No valid steps generated');
        }

        setState((prev) => ({
          ...prev,
          suggestions: validSuggestions,
          loading: false,
          error: null,
        }));

        return validSuggestions;
      } catch (err) {
        const error =
          err instanceof Error ? err.message : 'Failed to generate steps';
        setState((prev) => ({
          ...prev,
          error,
          loading: false,
          suggestions: [],
        }));
        throw err;
      }
    },
    [apiKeyAnthropic, validateStepType]
  );

  /**
   * Convert AI suggestion to ScenarioStep
   */
  const suggestionToStep = useCallback((suggestion: AiSuggestion): ScenarioStep => {
    return {
      id: `step_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      name: suggestion.name,
      type: suggestion.type as any,
      data: suggestion.data || {},
      enabled: true,
    };
  }, []);

  /**
   * Clear suggestions and errors
   */
  const clear = useCallback(() => {
    setState({
      loading: false,
      error: null,
      suggestions: [],
      lastQuery: '',
    });
  }, []);

  return {
    ...state,
    generateSteps,
    suggestionToStep,
    clear,
  };
};
