export class Candle {
    constructor(
        public open: number,
        public high: number,
        public low: number,
        public close: number,
        public volume: number,
        public time: Date
    ) {
    }
}