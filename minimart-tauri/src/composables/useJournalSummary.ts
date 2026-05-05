export interface SummaryLine {
  account_code?: string | null
  account_name?: string | null
  account_type?: string | null
  debit: number
  credit: number
}

export interface JournalBusinessSummary {
  transactionAmount: number
  vatAmount: number
  netSales: number
  cogs: number
  profit: number
  marginPercent: number
}

const isAssetCashLike = (line: SummaryLine) => {
  const code = (line.account_code || '').trim()
  const name = (line.account_name || '').toLowerCase()
  return line.account_type === 'Asset' && (
    ['1100', '1110', '1120'].includes(code) ||
    name.includes('cash') ||
    name.includes('m-pesa') ||
    name.includes('mpesa') ||
    name.includes('card')
  )
}

const isVatLine = (line: SummaryLine) => {
  const code = (line.account_code || '').trim()
  const name = (line.account_name || '').toLowerCase()
  return code === '2100' || name.includes('vat')
}

const isRevenueLine = (line: SummaryLine) => line.account_type === 'Revenue'

const isCogsLine = (line: SummaryLine) => {
  const code = (line.account_code || '').trim()
  const name = (line.account_name || '').toLowerCase()
  return code === '5100' || name.includes('cost of goods')
}

export const summarizeJournalEntry = (lines: SummaryLine[]): JournalBusinessSummary => {
  const transactionAmount = lines
    .filter(isAssetCashLike)
    .reduce((sum, line) => sum + Number(line.debit || 0), 0)

  const vatCredit = lines
    .filter(isVatLine)
    .reduce((sum, line) => sum + Number(line.credit || 0), 0)

  const vatAmount = vatCredit > 0 ? vatCredit : (transactionAmount * 16) / 116

  const netSales = lines
    .filter(isRevenueLine)
    .reduce((sum, line) => sum + Number(line.credit || 0), 0)

  const cogs = lines
    .filter(isCogsLine)
    .reduce((sum, line) => sum + Number(line.debit || 0), 0)

  const profit = netSales - cogs
  const marginPercent = netSales > 0 ? (profit / netSales) * 100 : 0

  return { transactionAmount, vatAmount, netSales, cogs, profit, marginPercent }
}

