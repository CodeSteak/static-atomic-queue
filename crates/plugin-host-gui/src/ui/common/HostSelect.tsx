import React, {useMemo} from "react";
import Select, {ActionMeta, GroupTypeBase} from "react-select";
import styled from "styled-components";
import {ACTIVE_BORDER_COLOR, BLACK, BORDER_COLOR, LIGHT_GRAY, MEDIUM_GRAY,} from "../constants";
import {StylesConfig} from "react-select/src/styles";

export const SelectWrapper = styled.div({
  userSelect: "none",
  alignItems: "center",
  display: "flex",
  marginBottom: 10,
});

export const SelectLabel = styled.div({
  userSelect: "none",
  minWidth: 150,
  textAlign: "right",
  fontWeight: "bold",
  marginRight: 10,
});

const SelectExpand = styled.div({
  flex: 1,
});

export type OptionType<T> = { label: string; value: T };

interface Props<T> {
  label: string;
  options: OptionType<T>[];
  value: T | null;
  onChange: (value: OptionType<T>) => void;
}

function getReactSelectStyles<T>(): StylesConfig<
  OptionType<T>,
  false,
  GroupTypeBase<OptionType<T>>
> {
  return {
    option: (base) => ({
      ...base,
      background: BLACK,
      border: `solid 1px ${MEDIUM_GRAY}`,
      borderBottom: `solid 1px ${BORDER_COLOR}`,
      "&:hover": {
        background: LIGHT_GRAY,
        border: `solid 1px ${ACTIVE_BORDER_COLOR}`,
      },
    }),
    control: (base, props) => ({
      ...base,
      background: BLACK,
      borderRadius: 0,
      border: props.menuIsOpen
        ? `solid 1px ${ACTIVE_BORDER_COLOR}`
        : `solid 1px ${BORDER_COLOR}`,
      outline: "none",
      "&hover": {
        border: props.menuIsOpen
          ? `solid 1px ${ACTIVE_BORDER_COLOR}`
          : `solid 1px ${BORDER_COLOR}`,
      },
    }),
    menu: (base) => ({
      ...base,
      borderRadius: 0,
      marginTop: 2,
      padding: 0,
    }),
    menuList: (base) => ({
      ...base,
      margin: 0,
      padding: 0,
      borderLeft: `solid 1px ${BORDER_COLOR}`,
      borderRight: `solid 1px ${BORDER_COLOR}`,
    }),
    valueContainer: (base) => ({
      ...base,
    }),
    singleValue: (base) => ({
      ...base,
      color: "white",
      margin: 0,
      padding: 0,
      "&:focus": {
        border: `solid 1px ${ACTIVE_BORDER_COLOR}`,
      },
    }),
  };
}

export function HostSelect<T>({ label, options, value, onChange }: Props<T>) {
  const optionValue: OptionType<T> =
    useMemo(
      () => options.find((option) => option.value === value),
      [value, options]
    ) ?? options[0];
  const reactSelectStyles = getReactSelectStyles();
  const onChangeCallback = (
    value: OptionType<T> | null,
    _meta: ActionMeta<OptionType<T>>
  ) => value != null && onChange(value);

  return (
    <SelectWrapper>
      <SelectLabel>{label}</SelectLabel>
      <SelectExpand>
        <Select
          value={optionValue}
          options={options}
          onChange={onChangeCallback}
          // @ts-ignore
          styles={reactSelectStyles}
        />
      </SelectExpand>
    </SelectWrapper>
  );
}