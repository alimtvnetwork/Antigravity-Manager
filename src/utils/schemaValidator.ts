/**
 * Live JSON Schema Parameter Validator
 * Validates request parameters and payloads against JSON schema definitions before network dispatch.
 */

export interface JsonSchemaProperty {
  type?: 'string' | 'number' | 'integer' | 'boolean' | 'object' | 'array' | 'null';
  required?: string[];
  properties?: Record<string, JsonSchemaProperty>;
  items?: JsonSchemaProperty;
  enum?: (string | number | boolean)[];
  minimum?: number;
  maximum?: number;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  description?: string;
}

export interface ValidationError {
  path: string;
  message: string;
  expected?: string;
  actual?: string;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
}

export function validateJsonSchema(value: unknown, schema: JsonSchemaProperty, path = '$'): ValidationResult {
  const errors: ValidationError[] = [];

  // Check type if specified
  if (schema.type) {
    const actualType = getJsonType(value);
    const typeMatches =
      schema.type === 'integer'
        ? typeof value === 'number' && Number.isInteger(value)
        : actualType === schema.type;

    if (!typeMatches) {
      errors.push({
        path,
        message: `Expected type '${schema.type}', received '${actualType}'`,
        expected: schema.type,
        actual: actualType,
      });
      return { isValid: false, errors };
    }
  }

  // String validations
  if (typeof value === 'string') {
    if (schema.minLength !== undefined && value.length < schema.minLength) {
      errors.push({
        path,
        message: `Length ${value.length} is less than minLength ${schema.minLength}`,
      });
    }
    if (schema.maxLength !== undefined && value.length > schema.maxLength) {
      errors.push({
        path,
        message: `Length ${value.length} exceeds maxLength ${schema.maxLength}`,
      });
    }
    if (schema.pattern) {
      try {
        const regex = new RegExp(schema.pattern);
        if (!regex.test(value)) {
          errors.push({
            path,
            message: `Value does not match pattern '${schema.pattern}'`,
          });
        }
      } catch {
        // Invalid regex pattern in schema ignored
      }
    }
  }

  // Number validations
  if (typeof value === 'number') {
    if (schema.minimum !== undefined && value < schema.minimum) {
      errors.push({
        path,
        message: `Value ${value} is less than minimum ${schema.minimum}`,
      });
    }
    if (schema.maximum !== undefined && value > schema.maximum) {
      errors.push({
        path,
        message: `Value ${value} exceeds maximum ${schema.maximum}`,
      });
    }
  }

  // Enum validations
  if (schema.enum && !schema.enum.includes(value as string | number | boolean)) {
    errors.push({
      path,
      message: `Value '${String(value)}' is not in allowed enum: [${schema.enum.join(', ')}]`,
    });
  }

  // Object validations
  if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
    const obj = value as Record<string, unknown>;

    // Required properties
    if (schema.required) {
      for (const reqKey of schema.required) {
        if (!(reqKey in obj) || obj[reqKey] === undefined) {
          errors.push({
            path: `${path}.${reqKey}`,
            message: `Missing required property '${reqKey}'`,
          });
        }
      }
    }

    // Nested properties
    if (schema.properties) {
      for (const [propKey, propSchema] of Object.entries(schema.properties)) {
        if (propKey in obj && obj[propKey] !== undefined) {
          const nestedResult = validateJsonSchema(obj[propKey], propSchema, `${path}.${propKey}`);
          errors.push(...nestedResult.errors);
        }
      }
    }
  }

  // Array validations
  if (Array.isArray(value) && schema.items) {
    value.forEach((item, index) => {
      const itemResult = validateJsonSchema(item, schema.items!, `${path}[${index}]`);
      errors.push(...itemResult.errors);
    });
  }

  return {
    isValid: errors.length === 0,
    errors,
  };
}

function getJsonType(val: unknown): string {
  if (val === null) return 'null';
  if (Array.isArray(val)) return 'array';
  return typeof val;
}
