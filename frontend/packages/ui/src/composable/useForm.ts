import { cloneDeep as _cloneDeep, get as _get, isEqual as _isEqual, set as _set } from "lodash-es";
import { computed, inject, markRaw, MaybeRefOrGetter, onMounted, provide, readonly, ref, Ref, toValue, watch } from "vue";
import { z } from "zod";

const FormProviderSymbol = Symbol("FormProvider");

export const IDLE = 0x00;
export const VALID = 0x01;
export const VALIDATING = 0x02;
export const SUBMITTING = 0x04;
export const SUBMITTED = 0x08;
export const SUBMIT_FAILED = 0x10;

export interface ProvideFormOptions<S extends z.ZodType> {
  schema: MaybeRefOrGetter<S>
  values: Ref<z.infer<S>>
  validateOnMount?: MaybeRefOrGetter<boolean>
  onSubmit?(values: z.output<S>): Promise<any>
}

export interface UseForm<T> {
  values: Ref<T>
  state: Ref<number>

  validate(): Promise<T>
  reset(): void
  submit(): Promise<void>

  get(key: string): any
  set(key: string, value: any): void
  setAndValidate(key: string, value: any): Promise<void>

  getError(key: string): string | null | undefined
  clearErrors(): void

  canSubmit(): boolean
  canReset(): boolean
  canSet(): boolean
  canValidate(): boolean
  validateField(key: string): Promise<void>
}

export function pathToKey(path: Array<string | number>) {
  let key = '';

  for (const part of path) {
    if (typeof part === 'number') {
      key += `[${part}]`;
    } else if (key) {
      key += `.${part}`;
    } else {
      key = part;
    }
  }

  return key;
}

export function provideForm<S extends z.ZodType>(options: ProvideFormOptions<S>): UseForm<z.output<S>> {
  const values = ref(toValue(options.values)) as Ref<z.output<S>>;

  let initialFormValues = _cloneDeep(values.value);

  const state = ref(IDLE);
  const errors = ref<Record<string, string>>({});

  watch(values, (newValues) => {
    if (!_isEqual(newValues, options.values.value)) {
      options.values.value = newValues;
    }
  }, { deep: true })

  function canSubmit() {
    return !!(state.value & VALID) && !(state.value & (VALIDATING | SUBMITTING));
  }

  function canReset() {
    return !(state.value & (VALIDATING | SUBMITTING));
  }

  function canSet() {
    return !(state.value & SUBMITTING);
  }

  function canValidate() {
    return !(state.value & (VALIDATING | SUBMITTING));
  }

  function clearErrors() {
    errors.value = {};
  }

  async function submit() {
    if (state.value & SUBMITTING) {
      throw new Error("Already submitting");
    }

    state.value &= ~SUBMIT_FAILED;
    state.value |= SUBMITTING;

    try {
      const values = await validate();

      if (state.value & VALID) {
        await options.onSubmit?.(values);
        state.value |= SUBMITTED;
      }
    } catch (error) {
      console.error(error);
      state.value |= SUBMIT_FAILED;
    } finally {
      state.value &= ~SUBMITTING;
    }
  }

  function set(key: string, value: any) {
    values.value = _set(values.value ?? {}, key, value);
  }

  async function validateField(key: string) {
    if (state.value & VALIDATING) {
      return;
    }

    delete errors.value[key];

    state.value |= VALIDATING;

    const schema = toValue(options.schema);

    const validationResult = await schema.safeParseAsync(values.value);

    try {
      if (validationResult.success) {
        state.value |= VALID;
        values.value = validationResult.data;
      } else {
        state.value &= ~VALID;

        for (const issue of validationResult.error.issues) {
          const errorKey = pathToKey(issue.path);

          if (errorKey === key) {
            errors.value[key] = issue.message;
            break
          }
        }
      }
    } finally {
      state.value &= ~VALIDATING;
    }
  }

  async function setAndValidate(key: string, value: any) {
    set(key, value);
    validateField(key);
  }

  function get(key: string) {
    return _get(values.value, key);
  }

  async function validate(setErrors = true) {
    if (state.value & VALIDATING) {
      throw new Error("Already validating");
    }

    state.value |= VALIDATING;

    const schema = toValue(options.schema);

    clearErrors();

    const validationResult = await schema.safeParseAsync(values.value);

    try {
      if (validationResult.success) {
        state.value |= VALID;
        return validationResult.data;
      } else {
        state.value &= ~VALID;

        if (setErrors) {
          errors.value = validationResult.error.issues.reduce((acc, error) => {
            const key = pathToKey(error.path);
            acc[key] = error.message;
            return acc;
          }, {} as Record<string, string>);
        }
      }
    } finally {
      state.value &= ~VALIDATING;
    }
  }

  async function reset() {
    if (state.value & SUBMITTING) {
      throw new Error("Cannot reset while submitting");
    }

    const schema = toValue(options.schema);

    const result = await schema.safeParseAsync(undefined);

    if (result.success) {
      values.value = result.data;
      state.value |= VALID;
    } else {
      const result = await schema.safeParseAsync(initialFormValues);

      if (result.success) {
        values.value = result.data;
        state.value |= VALID;
      } else {
        values.value = initialFormValues;
        state.value &= ~VALID;
      }
    }

    clearErrors();
  }

  function getError(key: string) {
    return errors.value[key] ?? null
  }



  onMounted(async () => {
    values.value = await validate(toValue(options.validateOnMount));
  })

  const provider: UseForm<z.output<S>> = markRaw({
    values,
    state: readonly(state),

    clearErrors,
    getError,

    get,
    set,
    setAndValidate,

    submit,
    reset,
    validate,

    canReset,
    canSet,
    canSubmit,
    canValidate,
    validateField
  })

  provide(FormProviderSymbol, provider);

  return provider
}

export function useForm<T>(): UseForm<T> | undefined {
  return inject<UseForm<T> | undefined>(FormProviderSymbol, undefined);
}

export interface UseFormFieldOptions<T> {
  value: Ref<T>
  key: MaybeRefOrGetter<string>
}

export interface useFormField<T> {
  value: Ref<T>
  error: Ref<string | null | undefined>
  canSet(): boolean
}

export function useFormField(options: UseFormFieldOptions<any>): useFormField<any> {
  const form = useForm();

  const value = computed({
    get() {
      const key = toValue(options.key);

      if (key && form) {
        return form.get(key);
      }

      return options.value.value;
    },
    set(value) {
      const key = toValue(options.key);

      if (key) {
        form?.setAndValidate(key, value);
      }

      options.value.value = value;
    }
  })

  watch(options.value, (newValue) => {
    if (!_isEqual(value.value, newValue)) {
      value.value = newValue;
    }
  })

  function canSet() {
    return form?.canSet() ?? true;
  }

  return {
    value,
    error: computed(() => form?.getError(toValue(options.key))),
    canSet
  }
}